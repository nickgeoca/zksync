//!
//! The semantic analyzer constant structure element.
//!

mod tests;

pub mod error;

use std::fmt;

use crate::lexical::token::location::Location;
use crate::semantic::element::access::dot::stack_field::StackField as StackFieldAccess;
use crate::semantic::element::constant::Constant;
use crate::semantic::element::r#type::structure::Structure as StructureType;
use crate::semantic::element::r#type::Type;
use crate::syntax::tree::identifier::Identifier;

use self::error::Error;

///
/// Structures are collections of named elements of different types.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Structure {
    pub location: Location,
    pub r#type: Option<StructureType>,
    pub values: Vec<(Identifier, Constant)>,
    pub is_validated: bool,
}

impl Structure {
    pub fn new(location: Location) -> Self {
        Self {
            location,
            r#type: None,
            values: vec![],
            is_validated: false,
        }
    }

    pub fn r#type(&self) -> Type {
        self.r#type
            .clone()
            .map(Type::Structure)
            .expect(crate::panic::VALIDATED_DURING_SEMANTIC_ANALYSIS)
    }

    pub fn has_the_same_type_as(&self, other: &Self) -> bool {
        self.r#type == other.r#type
    }

    pub fn push(&mut self, identifier: Identifier, value: Constant) {
        self.values.push((identifier, value));
    }

    pub fn validate(&mut self, expected: StructureType) -> Result<(), Error> {
        for (index, (identifier, constant)) in self.values.iter().enumerate() {
            match expected.fields.get(index) {
                Some((expected_name, expected_type)) => {
                    if &identifier.name != expected_name {
                        return Err(Error::FieldExpected {
                            location: identifier.location,
                            type_identifier: expected.identifier.to_owned(),
                            position: index + 1,
                            expected: expected_name.to_owned(),
                            found: identifier.name.to_owned(),
                        });
                    }

                    let r#type = constant.r#type();
                    if &r#type != expected_type {
                        return Err(Error::FieldInvalidType {
                            location: constant.location(),
                            type_identifier: expected.identifier.to_owned(),
                            field_name: expected_name.to_owned(),
                            expected: expected_type.to_string(),
                            found: r#type.to_string(),
                        });
                    }
                }
                None => {
                    return Err(Error::FieldOutOfRange {
                        location: identifier.location,
                        type_identifier: expected.identifier.to_owned(),
                        expected: expected.fields.len(),
                        found: index + 1,
                    });
                }
            }
        }

        self.r#type = Some(expected);
        self.is_validated = true;

        Ok(())
    }

    pub fn slice(self, identifier: Identifier) -> Result<(Constant, StackFieldAccess), Error> {
        let mut offset = 0;
        let total_size = self.r#type().size();

        for (index, (name, value)) in self.values.into_iter().enumerate() {
            let element_size = value.r#type().size();

            if name.name == identifier.name {
                let access =
                    StackFieldAccess::new(name.name, index, offset, element_size, total_size);

                return Ok((value, access));
            }

            offset += element_size;
        }

        Err(Error::FieldDoesNotExist {
            location: identifier.location,
            type_identifier: self
                .r#type
                .expect(crate::panic::VALIDATED_DURING_SEMANTIC_ANALYSIS)
                .identifier,
            field_name: identifier.name,
        })
    }
}

impl fmt::Display for Structure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' with fields {{ {} }}",
            self.r#type
                .as_ref()
                .expect(crate::panic::VALIDATED_DURING_SEMANTIC_ANALYSIS)
                .identifier,
            self.values
                .iter()
                .map(|(identifier, value)| format!("{}: {}", identifier.name, value))
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}
