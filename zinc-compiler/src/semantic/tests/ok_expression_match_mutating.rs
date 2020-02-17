//!
//! A semantic analyzer test.
//!

#![cfg(test)]

use num_bigint::BigInt;

use zinc_bytecode::scalar::IntegerType;
use zinc_bytecode::Call;
use zinc_bytecode::Else;
use zinc_bytecode::EndIf;
use zinc_bytecode::Eq;
use zinc_bytecode::Exit;
use zinc_bytecode::If;
use zinc_bytecode::Instruction;
use zinc_bytecode::Load;
use zinc_bytecode::PushConst;
use zinc_bytecode::Return;
use zinc_bytecode::Store;

#[test]
fn test() {
    let input = r#"
fn main() {
    let mut result = 0;
    let value = 2;
    match value {
        1 => 1,
        _ => {
            result = 42;
            2
        },
    };
}
"#;

    let expected = Ok(vec![
        Instruction::Call(Call::new(2, 0)),
        Instruction::Exit(Exit::new(0)),
        Instruction::PushConst(PushConst::new(BigInt::from(0), IntegerType::U8.into())),
        Instruction::Store(Store::new(0)),
        Instruction::PushConst(PushConst::new(BigInt::from(2), IntegerType::U8.into())),
        Instruction::Store(Store::new(1)),
        Instruction::Load(Load::new(1)),
        Instruction::Store(Store::new(2)),
        Instruction::Load(Load::new(2)),
        Instruction::PushConst(PushConst::new(BigInt::from(1), IntegerType::U8.into())),
        Instruction::Eq(Eq),
        Instruction::If(If),
        Instruction::PushConst(PushConst::new(BigInt::from(1), IntegerType::U8.into())),
        Instruction::Else(Else),
        Instruction::PushConst(PushConst::new(BigInt::from(42), IntegerType::U8.into())),
        Instruction::Store(Store::new(0)),
        Instruction::PushConst(PushConst::new(BigInt::from(2), IntegerType::U8.into())),
        Instruction::EndIf(EndIf),
        Instruction::Return(Return::new(0)),
    ]);

    let result = super::get_instructions(input);

    assert_eq!(result, expected);
}