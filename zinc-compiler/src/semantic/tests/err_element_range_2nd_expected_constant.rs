//!
//! A semantic analyzer test.
//!

#![cfg(test)]

use crate::lexical::Location;

use crate::semantic::Element;
use crate::semantic::ElementError;
use crate::semantic::Error as SemanticError;
use crate::semantic::Type;

use crate::Error;

#[test]
fn test() {
    let input = r#"
fn main() {
    let b = 42;
    0 .. b
}
"#;

    let expected = Err(Error::Semantic(SemanticError::Element(
        Location::new(4, 7),
        ElementError::OperatorRangeSecondOperandExpectedConstant(
            Element::Type(Type::new_integer_unsigned(crate::BITLENGTH_BYTE)).to_string(),
        ),
    )));

    let result = super::get_binary_result(input);

    assert_eq!(expected, result);
}
