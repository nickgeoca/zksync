// Built-in deps
use std::str::FromStr;
// External imports
use bigdecimal::BigDecimal;
use diesel::dsl::{insert_into, update};
use diesel::prelude::*;
use web3::types::{H256, U256};
// Workspace imports
use models::{
    ethereum::{ETHOperation, OperationType},
    Operation,
};
// Local imports
use self::records::{
    ETHBinding, ETHNonce, ETHStats, ETHTxHash, NewETHBinding, NewETHOperation, NewETHTxHash,
    StorageETHOperation,
};
use crate::chain::operations::records::StoredOperation;
use crate::schema::*;
use crate::StorageProcessor;

pub mod records;

/// Ethereum schema is capable of storing the information about the
/// interaction with the Ethereum blockchain (mainly the list of sent
/// Ethereum transactions).
#[derive(Debug)]
pub struct EthereumSchema<'a>(pub &'a StorageProcessor);

impl<'a> EthereumSchema<'a> {
    /// Loads the list of operations that were not confirmed on Ethereum,
    /// each operation has a list of sent Ethereum transactions.
    pub fn load_unconfirmed_operations(
        &self,
    ) -> QueryResult<Vec<(ETHOperation, Option<Operation>)>> {
        // Load the operations with the associated Ethereum transactions
        // from the database.
        // Here we obtain a sequence of one-to-one mappings (ETH tx) -> (operation ID).
        // Each Ethereum transaction can have no more than one associated operation, and each
        // operation is associated with exactly one Ethereum transaction. Note that there may
        // be ETH transactions without an operation (e.g. `completeWithdrawals` call), but for
        // every operation always there is an ETH transaction.
        let raw_ops: Vec<(
            StorageETHOperation,
            Option<ETHBinding>,
            Option<StoredOperation>,
        )> = self.0.conn().transaction(|| {
            eth_operations::table
                .left_join(
                    eth_ops_binding::table.on(eth_operations::id.eq(eth_ops_binding::eth_op_id)),
                )
                .left_join(operations::table.on(operations::id.eq(eth_ops_binding::op_id)))
                .filter(eth_operations::confirmed.eq(false))
                .order(eth_operations::id.asc())
                .load(self.0.conn())
        })?;

        // Create a vector for the expected output.
        let mut ops: Vec<(ETHOperation, Option<Operation>)> = Vec::with_capacity(raw_ops.len());

        // Transform the `StoredOperation` to `Operation` and `StoredETHOperation` to `ETHOperation`.
        for (eth_op, _, raw_op) in raw_ops {
            // Load the stored txs hashes.
            let eth_tx_hashes: Vec<ETHTxHash> = eth_tx_hashes::table
                .filter(eth_tx_hashes::eth_op_id.eq(eth_op.id))
                .load(self.0.conn())?;
            assert!(
                eth_tx_hashes.len() >= 1,
                "No hashes stored for the Ethereum operation"
            );

            // If there is an operation, convert it to the `Operation` type.
            let op = if let Some(raw_op) = raw_op {
                Some(raw_op.into_op(self.0)?)
            } else {
                None
            };

            // Convert the fields into expected format.
            let op_type = OperationType::from_str(eth_op.op_type.as_ref())
                .expect("Stored operation type must have a valid value");
            let last_used_gas_price =
                U256::from_str(&eth_op.last_used_gas_price.to_string()).unwrap();
            let used_tx_hashes = eth_tx_hashes
                .iter()
                .map(|entry| H256::from_slice(&entry.tx_hash))
                .collect();
            let final_hash = eth_op.final_hash.map(|hash| H256::from_slice(&hash));

            let eth_op = ETHOperation {
                id: eth_op.id,
                op_type,
                nonce: eth_op.nonce.into(),
                last_deadline_block: eth_op.last_deadline_block,
                last_used_gas_price,
                used_tx_hashes,
                encoded_tx_data: eth_op.raw_tx,
                confirmed: eth_op.confirmed,
                final_hash,
            };

            ops.push((eth_op, op));
        }

        Ok(ops)
    }

    /// Stores the sent (but not confirmed yet) Ethereum transaction in the database.
    pub fn save_new_eth_tx(
        &self,
        op_type: OperationType,
        op_id: Option<i64>,
        hash: H256,
        deadline_block: u64,
        nonce: u32,
        gas_price: BigDecimal,
        raw_tx: Vec<u8>,
    ) -> QueryResult<()> {
        let operation = NewETHOperation {
            op_type: op_type.to_string(),
            nonce: i64::from(nonce),
            last_deadline_block: deadline_block as i64,
            last_used_gas_price: gas_price,
            raw_tx,
        };

        self.0.conn().transaction(|| {
            // Insert the operation itself.
            let inserted_tx = insert_into(eth_operations::table)
                .values(&operation)
                .returning(eth_operations::id)
                .get_results(self.0.conn())?;
            assert_eq!(
                inserted_tx.len(),
                1,
                "Wrong amount of updated rows (eth_operations)"
            );

            // Obtain the operation ID for the follow-up queried.
            let eth_op_id = inserted_tx[0];

            // Add a hash entry.
            let hash_entry = NewETHTxHash {
                eth_op_id,
                tx_hash: hash.as_bytes().to_vec(),
            };
            let inserted_hashes_rows = insert_into(eth_tx_hashes::table)
                .values(&hash_entry)
                .execute(self.0.conn())?;
            assert_eq!(
                inserted_hashes_rows, 1,
                "Wrong amount of updated rows (eth_tx_hashes)"
            );

            // If the operation ID was provided, we should also insert a binding entry.
            if let Some(op_id) = op_id {
                let binding = NewETHBinding { op_id, eth_op_id };

                insert_into(eth_ops_binding::table)
                    .values(&binding)
                    .execute(self.0.conn())?;
            }

            self.report_created_operation(op_type)?;

            Ok(())
        })
    }

    /// Retrieves the Ethereum operation ID given the tx hash.
    fn get_eth_op_id(&self, hash: &H256) -> QueryResult<i64> {
        let hash_entry = eth_tx_hashes::table
            .filter(eth_tx_hashes::tx_hash.eq(hash.as_bytes()))
            .first::<ETHTxHash>(self.0.conn())?;

        Ok(hash_entry.eth_op_id)
    }

    /// Updates the Ethereum operation by adding a new tx data.
    /// The new deadline block / gas value are placed instead of old values to the main entry,
    /// and for hash a new `eth_tx_hashes` entry is added.
    pub fn update_eth_tx(
        &self,
        eth_op_id: i64,
        hash: &H256,
        new_deadline_block: i64,
        new_gas_value: BigDecimal,
    ) -> QueryResult<()> {
        self.0.conn().transaction(|| {
            // Insert the new hash entry.
            let hash_entry = NewETHTxHash {
                eth_op_id,
                tx_hash: hash.as_bytes().to_vec(),
            };
            let inserted_hashes_rows = insert_into(eth_tx_hashes::table)
                .values(&hash_entry)
                .execute(self.0.conn())?;
            assert_eq!(
                inserted_hashes_rows, 1,
                "Wrong amount of updated rows (eth_tx_hashes)"
            );

            // Update the stored tx.
            update(eth_operations::table.filter(eth_operations::id.eq(eth_op_id)))
                .set((
                    eth_operations::last_used_gas_price.eq(new_gas_value),
                    eth_operations::last_deadline_block.eq(new_deadline_block),
                ))
                .execute(self.0.conn())?;

            Ok(())
        })
    }

    /// Updates the stats counter with the new operation reported.
    /// This method should be called once **per operation**. It means that if transaction
    /// for some operation was stuck, and another transaction was created for it, this method
    /// **should not** be invoked.
    ///
    /// This method expects the database to be initially prepared with inserting the actual
    /// nonce value. Currently the script `db-insert-eth-data.sh` is responsible for that
    /// and it's invoked within `db-reset` subcommand.
    fn report_created_operation(&self, operation_type: OperationType) -> QueryResult<()> {
        self.0.conn().transaction(|| {
            let mut current_stats: ETHStats = eth_stats::table.first(self.0.conn())?;

            // Increase the only one type of operations.
            match operation_type {
                OperationType::Commit => {
                    current_stats.commit_ops += 1;
                }
                OperationType::Verify => {
                    current_stats.verify_ops += 1;
                }
                OperationType::Withdraw => {
                    current_stats.withdraw_ops += 1;
                }
            };

            // Update the stored stats.
            update(eth_stats::table.filter(eth_stats::id.eq(true)))
                .set((
                    eth_stats::commit_ops.eq(current_stats.commit_ops),
                    eth_stats::verify_ops.eq(current_stats.verify_ops),
                    eth_stats::withdraw_ops.eq(current_stats.withdraw_ops),
                ))
                .execute(self.0.conn())?;

            Ok(())
        })
    }

    /// Loads the stored Ethereum operations stats.
    pub fn load_stats(&self) -> QueryResult<ETHStats> {
        eth_stats::table.first(self.0.conn())
    }

    /// Marks the stored Ethereum transaction as confirmed (and thus the associated `Operation`
    /// is marked as confirmed as well).
    pub fn confirm_eth_tx(&self, hash: &H256) -> QueryResult<()> {
        self.0.conn().transaction(|| {
            let eth_op_id = self.get_eth_op_id(hash)?;

            // Set the `confirmed` and `final_hash` field of the entry.
            let updated: Vec<i64> =
                update(eth_operations::table.filter(eth_operations::id.eq(eth_op_id)))
                    .set((
                        eth_operations::confirmed.eq(true),
                        eth_operations::final_hash.eq(Some(hash.as_bytes().to_vec())),
                    ))
                    .returning(eth_operations::id)
                    .get_results(self.0.conn())?;

            assert_eq!(
                updated.len(),
                1,
                "Unexpected amount of operations were confirmed"
            );

            let eth_op_id = updated[0];

            let binding: Option<ETHBinding> = eth_ops_binding::table
                .filter(eth_ops_binding::eth_op_id.eq(eth_op_id))
                .first::<ETHBinding>(self.0.conn())
                .optional()?;

            // If there is a ZKSync operation, mark it as confirmed as well.
            if let Some(binding) = binding {
                let op = operations::table
                    .filter(operations::id.eq(binding.op_id))
                    .first::<StoredOperation>(self.0.conn())?;

                update(operations::table.filter(operations::id.eq(op.id)))
                    .set(operations::confirmed.eq(true))
                    .execute(self.0.conn())
                    .map(drop)?;
            }

            Ok(())
        })
    }

    /// Obtains the next nonce to use and updates the corresponding entry in the database
    /// for the next invocation.
    ///
    /// This method expects the database to be initially prepared with inserting the actual
    /// nonce value. Currently the script `db-insert-eth-data.sh` is responsible for that
    /// and it's invoked within `db-reset` subcommand.
    pub fn get_next_nonce(&self) -> QueryResult<i64> {
        let old_nonce: ETHNonce = eth_nonce::table.first(self.0.conn())?;

        let new_nonce_value = old_nonce.nonce + 1;

        update(eth_nonce::table.filter(eth_nonce::id.eq(true)))
            .set(eth_nonce::nonce.eq(new_nonce_value))
            .execute(self.0.conn())?;

        let old_nonce_value = old_nonce.nonce;

        Ok(old_nonce_value)
    }

    /// Method that internally initializes the `eth_nonce` and `eth_stats` tables.
    /// Since in db tests the database is empty, we must provide a possibility
    /// to initialize required db fields.
    #[cfg(test)]
    pub fn initialize_eth_data(&self) -> QueryResult<()> {
        #[derive(Debug, Insertable)]
        #[table_name = "eth_nonce"]
        pub struct NewETHNonce {
            pub nonce: i64,
        }

        #[derive(Debug, Insertable)]
        #[table_name = "eth_stats"]
        pub struct NewETHStats {
            pub commit_ops: i64,
            pub verify_ops: i64,
            pub withdraw_ops: i64,
        }

        let old_nonce: Option<ETHNonce> = eth_nonce::table.first(self.0.conn()).optional()?;

        if old_nonce.is_none() {
            // There is no nonce, we have to insert it manually.
            let nonce = NewETHNonce { nonce: 0 };

            insert_into(eth_nonce::table)
                .values(&nonce)
                .execute(self.0.conn())?;
        }

        let old_stats: Option<ETHStats> = eth_stats::table.first(self.0.conn()).optional()?;

        if old_stats.is_none() {
            let stats = NewETHStats {
                commit_ops: 0,
                verify_ops: 0,
                withdraw_ops: 0,
            };

            insert_into(eth_stats::table)
                .values(&stats)
                .execute(self.0.conn())?;
        }

        Ok(())
    }
}
