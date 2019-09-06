//!
//! The operator expression parser.
//!

mod add_sub;
mod and;
mod assignment;
mod casting;
mod comparison;
mod mul_div_rem;
mod or;
mod xor;

pub use self::add_sub::Parser as AddSubOperandParser;
pub use self::and::Parser as AndOperandParser;
pub use self::assignment::Parser as AssignmentOperandParser;
pub use self::casting::Parser as CastingOperandParser;
pub use self::comparison::Parser as ComparisonOperandParser;
pub use self::mul_div_rem::Parser as MulDivRemOperandParser;
pub use self::or::Parser as OrOperandParser;
pub use self::xor::Parser as XorOperandParser;

use std::cell::RefCell;
use std::rc::Rc;

use crate::lexical::Lexeme;
use crate::lexical::Symbol;
use crate::lexical::Token;
use crate::lexical::TokenStream;
use crate::syntax::OperatorExpression;
use crate::syntax::OperatorExpressionOperator;
use crate::Error;

#[derive(Debug, Clone, Copy)]
pub enum State {
    AssignmentFirstOperand,
    AssignmentOperator,
    AssignmentSecondOperand,
}

impl Default for State {
    fn default() -> Self {
        State::AssignmentFirstOperand
    }
}

#[derive(Default)]
pub struct Parser {
    state: State,
    expression: OperatorExpression,
    operator: Option<(OperatorExpressionOperator, Token)>,
}

impl Parser {
    pub fn parse(mut self, stream: Rc<RefCell<TokenStream>>) -> Result<OperatorExpression, Error> {
        loop {
            match self.state {
                State::AssignmentFirstOperand => {
                    let rpn = AssignmentOperandParser::default().parse(stream.clone())?;
                    self.expression.append(rpn);
                    self.state = State::AssignmentOperator;
                }
                State::AssignmentOperator => {
                    let peek = stream.borrow_mut().peek();
                    match peek {
                        Some(Ok(
                            token @ Token {
                                lexeme: Lexeme::Symbol(Symbol::Equals),
                                ..
                            },
                        )) => {
                            stream.borrow_mut().next();
                            self.operator = Some((OperatorExpressionOperator::Assignment, token));
                            self.state = State::AssignmentSecondOperand;
                        }
                        _ => return Ok(self.expression),
                    }
                }
                State::AssignmentSecondOperand => {
                    let rpn = AssignmentOperandParser::default().parse(stream.clone())?;
                    self.expression.append(rpn);
                    if let Some(operator) = self.operator.take() {
                        self.expression.push_operator(operator);
                    }
                    return Ok(self.expression);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::Parser;
    use crate::lexical::BooleanLiteral;
    use crate::lexical::Lexeme;
    use crate::lexical::Literal;
    use crate::lexical::Location;
    use crate::lexical::Symbol;
    use crate::lexical::Token;
    use crate::lexical::TokenStream;
    use crate::syntax::OperatorExpression;
    use crate::syntax::OperatorExpressionElement;
    use crate::syntax::OperatorExpressionObject;
    use crate::syntax::OperatorExpressionOperand;
    use crate::syntax::OperatorExpressionOperator;

    #[test]
    fn ok() {
        let code = r#"true || false"#;

        let expected = OperatorExpression::new(vec![
            OperatorExpressionElement::new(
                OperatorExpressionObject::Operand(OperatorExpressionOperand::Literal(
                    Literal::Boolean(BooleanLiteral::True),
                )),
                Token::new(
                    Lexeme::Literal(Literal::Boolean(BooleanLiteral::True)),
                    Location::new(1, 1),
                ),
            ),
            OperatorExpressionElement::new(
                OperatorExpressionObject::Operand(OperatorExpressionOperand::Literal(
                    Literal::Boolean(BooleanLiteral::False),
                )),
                Token::new(
                    Lexeme::Literal(Literal::Boolean(BooleanLiteral::False)),
                    Location::new(1, 9),
                ),
            ),
            OperatorExpressionElement::new(
                OperatorExpressionObject::Operator(OperatorExpressionOperator::Or),
                Token::new(
                    Lexeme::Symbol(Symbol::DoubleVerticalBar),
                    Location::new(1, 6),
                ),
            ),
        ]);

        let result = Parser::default()
            .parse(Rc::new(RefCell::new(TokenStream::new(code.to_owned()))))
            .expect("Syntax error");

        assert_eq!(expected, result);
    }
}
