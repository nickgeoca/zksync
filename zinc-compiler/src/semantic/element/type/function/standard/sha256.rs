//!
//! The semantic analyzer standard library `sha256` function type element.
//!

use std::fmt;
use std::ops::Deref;

use zinc_bytecode::builtins::BuiltinIdentifier;

use crate::semantic::element::r#type::function::standard::error::Error;
use crate::semantic::element::r#type::Type;

#[derive(Debug, Default, Clone)]
pub struct Function {
    identifier: &'static str,
    return_type: Box<Type>,
}

impl Function {
    const ARGUMENT_INDEX_PREIMAGE: usize = 0;
    const ARGUMENT_COUNT: usize = 1;

    pub fn new() -> Self {
        Self {
            identifier: "sha256",
            return_type: Box::new(Type::array(Type::boolean(), crate::SHA256_HASH_SIZE_BITS)),
        }
    }

    pub fn identifier(&self) -> &'static str {
        self.identifier
    }

    pub fn builtin_identifier(&self) -> BuiltinIdentifier {
        BuiltinIdentifier::CryptoSha256
    }

    pub fn validate(&self, inputs: &[Type]) -> Result<Type, Error> {
        let result = match inputs.get(Self::ARGUMENT_INDEX_PREIMAGE) {
            Some(Type::Array { r#type, size }) => match (r#type.deref(), *size) {
                (Type::Boolean, size) if size % crate::BITLENGTH_BYTE == 0 => {
                    Ok(self.return_type.deref().to_owned())
                }
                (r#type, size) => Err(Error::ArgumentType(
                    self.identifier,
                    "[bool; {8*N}]".to_owned(),
                    format!("[{}; {}]", r#type, size),
                )),
            },
            Some(r#type) => Err(Error::ArgumentType(
                self.identifier,
                "[bool; {8*N}]".to_owned(),
                r#type.to_string(),
            )),
            None => Err(Error::ArgumentCount(
                self.identifier,
                Self::ARGUMENT_COUNT,
                inputs.len(),
            )),
        };

        if inputs.get(Self::ARGUMENT_COUNT).is_some() {
            return Err(Error::ArgumentCount(
                self.identifier,
                Self::ARGUMENT_COUNT,
                inputs.len(),
            ));
        }

        result
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "fn std::{}(preimage: [bool: 8*N]) -> {}",
            self.identifier, self.return_type,
        )
    }
}