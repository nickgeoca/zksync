//!
//! The virtual machine contract output transfer.
//!

use num::BigUint;

///
/// The virtual machine contract output transfer.
///
#[derive(Debug)]
pub struct Transfer {
    /// The recepient address.
    pub recipient: [u8; zinc_const::size::ETH_ADDRESS],
    /// The zkSync address of the token being transferred.
    pub token_address: BigUint,
    /// The amount of the tokens being sent.
    pub amount: BigUint,
}

impl Transfer {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        recipient: [u8; zinc_const::size::ETH_ADDRESS],
        token_address: BigUint,
        amount: BigUint,
    ) -> Self {
        Self {
            recipient,
            token_address,
            amount,
        }
    }
}
