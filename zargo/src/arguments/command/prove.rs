//!
//! The Zargo package manager `prove` subcommand.
//!

use std::convert::TryFrom;
use std::path::PathBuf;

use failure::Fail;
use structopt::StructOpt;

use crate::arguments::command::IExecutable;
use crate::error::file::Error as FileError;
use crate::executable::virtual_machine::Error as VirtualMachineError;
use crate::executable::virtual_machine::VirtualMachine;
use crate::project::build::Directory as BuildDirectory;
use crate::project::data::private_key::PrivateKey as PrivateKeyFile;
use crate::project::data::Directory as DataDirectory;
use crate::project::manifest::project_type::ProjectType;
use crate::project::manifest::Manifest as ManifestFile;

///
/// The Zargo package manager `prove` subcommand.
///
#[derive(Debug, StructOpt)]
#[structopt(about = "Generates the zero-knowledge proof for given witness data")]
pub struct Command {
    /// Prints more logs, if passed several times.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: usize,

    /// The path to the Zargo project manifest file.
    #[structopt(
        long = "manifest-path",
        parse(from_os_str),
        default_value = zinc_const::path::MANIFEST,
    )]
    pub manifest_path: PathBuf,

    /// The contract method to prove. Only for contracts.
    #[structopt(long = "method")]
    pub method: Option<String>,
}

///
/// The Zargo package manager `prove` subcommand error.
///
#[derive(Debug, Fail)]
pub enum Error {
    /// The manifest file error.
    #[fail(display = "manifest file {}", _0)]
    ManifestFile(FileError<toml::de::Error>),
    /// The contract method to call is missing.
    #[fail(display = "contract method to call must be specified")]
    MethodMissing,
    /// The private key file generation error.
    #[fail(display = "private key file {}", _0)]
    PrivateKeyFile(FileError),
    /// The virtual machine process error.
    #[fail(display = "virtual machine {}", _0)]
    VirtualMachine(VirtualMachineError),
}

impl IExecutable for Command {
    type Error = Error;

    fn execute(self) -> Result<(), Self::Error> {
        let manifest = ManifestFile::try_from(&self.manifest_path).map_err(Error::ManifestFile)?;

        match manifest.project.r#type {
            ProjectType::Contract if self.method.is_none() => return Err(Error::MethodMissing),
            _ => {}
        }

        let mut manifest_path = self.manifest_path.clone();
        if manifest_path.is_file() {
            manifest_path.pop();
        }

        let data_directory_path = DataDirectory::path(&manifest_path);
        let mut witness_path = data_directory_path.clone();
        let mut public_data_path = data_directory_path.clone();
        if let Some(ref method) = self.method {
            witness_path.push(format!(
                "{}_{}.{}",
                zinc_const::file_name::WITNESS,
                method,
                zinc_const::extension::JSON,
            ));
            public_data_path.push(format!(
                "{}_{}.{}",
                zinc_const::file_name::PUBLIC_DATA,
                method,
                zinc_const::extension::JSON,
            ));

            if !PrivateKeyFile::exists_at(&data_directory_path) {
                PrivateKeyFile::default()
                    .write_to(&data_directory_path)
                    .map_err(Error::PrivateKeyFile)?;
            }
        } else {
            witness_path.push(format!(
                "{}.{}",
                zinc_const::file_name::WITNESS,
                zinc_const::extension::JSON,
            ));
            public_data_path.push(format!(
                "{}.{}",
                zinc_const::file_name::PUBLIC_DATA,
                zinc_const::extension::JSON,
            ));
        }
        let mut storage_path = data_directory_path.clone();
        storage_path.push(format!(
            "{}.{}",
            zinc_const::file_name::STORAGE,
            zinc_const::extension::JSON
        ));
        let mut proving_key_path = data_directory_path;
        proving_key_path.push(zinc_const::file_name::PROVING_KEY);

        let build_directory_path = BuildDirectory::path(&manifest_path);
        let mut binary_path = build_directory_path;
        binary_path.push(format!(
            "{}.{}",
            zinc_const::file_name::BINARY,
            zinc_const::extension::BINARY
        ));

        match self.method {
            Some(method) => VirtualMachine::prove_contract(
                self.verbosity,
                &binary_path,
                &proving_key_path,
                &witness_path,
                &public_data_path,
                &storage_path,
                method.as_str(),
            ),
            None => VirtualMachine::prove_circuit(
                self.verbosity,
                &binary_path,
                &proving_key_path,
                &witness_path,
                &public_data_path,
            ),
        }
        .map_err(Error::VirtualMachine)?;

        Ok(())
    }
}