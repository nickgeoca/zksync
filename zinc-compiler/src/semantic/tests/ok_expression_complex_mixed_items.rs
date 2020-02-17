//!
//! A semantic analyzer test.
//!

#![cfg(test)]

use num_bigint::BigInt;

use zinc_bytecode::scalar::{IntegerType, ScalarType};
use zinc_bytecode::Add;
use zinc_bytecode::Call;
use zinc_bytecode::Cast;
use zinc_bytecode::Exit;
use zinc_bytecode::Instruction;
use zinc_bytecode::Load;
use zinc_bytecode::PushConst;
use zinc_bytecode::Return;
use zinc_bytecode::Store;

#[test]
fn test() {
    let input = r#"
static STATIC: field = 5;

const CONST: field = 42;

fn main() -> field {
    let var: field = 69;

    STATIC + CONST + var
}
"#;

    let expected = Ok(vec![
        Instruction::Call(Call::new(2, 0)),
        Instruction::Exit(Exit::new(1)),
        Instruction::PushConst(PushConst::new(BigInt::from(69), IntegerType::U8.into())),
        Instruction::Cast(Cast::new(ScalarType::Field)),
        Instruction::Store(Store::new(0)),
        Instruction::PushConst(PushConst::new(BigInt::from(5), ScalarType::Field)),
        Instruction::PushConst(PushConst::new(BigInt::from(42), ScalarType::Field)),
        Instruction::Add(Add),
        Instruction::Load(Load::new(0)),
        Instruction::Add(Add),
        Instruction::Return(Return::new(1)),
    ]);

    let result = super::get_instructions(input);

    assert_eq!(result, expected);
}