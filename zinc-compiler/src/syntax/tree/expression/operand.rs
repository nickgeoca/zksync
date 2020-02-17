//!
//! The expression operand.
//!

use crate::syntax::ArrayExpression;
use crate::syntax::BlockExpression;
use crate::syntax::BooleanLiteral;
use crate::syntax::ConditionalExpression;
use crate::syntax::Expression;
use crate::syntax::Identifier;
use crate::syntax::IntegerLiteral;
use crate::syntax::MatchExpression;
use crate::syntax::MemberInteger;
use crate::syntax::MemberString;
use crate::syntax::StringLiteral;
use crate::syntax::StructureExpression;
use crate::syntax::TupleExpression;
use crate::syntax::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Unit,
    LiteralBoolean(BooleanLiteral),
    LiteralInteger(IntegerLiteral),
    LiteralString(StringLiteral),
    MemberInteger(MemberInteger),
    MemberString(MemberString),
    Identifier(Identifier),
    Type(Type),
    ExpressionList(Vec<Expression>),
    Block(BlockExpression),
    Conditional(ConditionalExpression),
    Match(MatchExpression),
    Array(ArrayExpression),
    Tuple(TupleExpression),
    Structure(StructureExpression),
}