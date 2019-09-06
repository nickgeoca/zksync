//!
//! The expression.
//!

mod block;
mod conditional;
mod operator;

pub use self::block::Expression as BlockExpression;
pub use self::conditional::Builder as ConditionalExpressionBuilder;
pub use self::conditional::Expression as ConditionalExpression;
pub use self::operator::Element as OperatorExpressionElement;
pub use self::operator::Expression as OperatorExpression;
pub use self::operator::Object as OperatorExpressionObject;
pub use self::operator::Operand as OperatorExpressionOperand;
pub use self::operator::Operator as OperatorExpressionOperator;

use std::fmt;

use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Expression {
    Operator(OperatorExpression),
    Block(BlockExpression),
    Conditional(ConditionalExpression),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Operator(expression) => write!(f, "( {} )", expression),
            Self::Block(expression) => write!(f, "{}", expression),
            Self::Conditional(expression) => write!(f, "{}", expression),
        }
    }
}
