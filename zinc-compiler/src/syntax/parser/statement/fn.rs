//!
//! The `fn` statement parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::lexical::stream::TokenStream;
use crate::lexical::token::lexeme::keyword::Keyword;
use crate::lexical::token::lexeme::symbol::Symbol;
use crate::lexical::token::lexeme::Lexeme;
use crate::lexical::token::Token;
use crate::syntax::error::Error as SyntaxError;
use crate::syntax::parser::expression::terminal::block::Parser as BlockExpressionParser;
use crate::syntax::parser::pattern_binding_list::Parser as BindingPatternListParser;
use crate::syntax::parser::r#type::Parser as TypeParser;
use crate::syntax::tree::identifier::Identifier;
use crate::syntax::tree::statement::r#fn::builder::Builder as FnStatementBuilder;

/// The missing identifier error hint.
pub static HINT_EXPECTED_IDENTIFIER: &str =
    "function must have an identifier, e.g. `fn sum(...) { ... }`";
/// The missing argument list error hint.
pub static HINT_EXPECTED_ARGUMENT_LIST: &str =
    "function must have the argument list, e.g. `fn sum(a: u8, b: u8) { ... }`";

///
/// The parser state.
///
#[derive(Debug, Clone, Copy)]
pub enum State {
    /// The initial state.
    KeywordFn,
    /// The `fn` has been parsed so far.
    Identifier,
    /// The `fn {identifier}` has been parsed so far.
    ParenthesisLeft,
    /// The `fn {identifier} (` has been parsed so far.
    ArgumentBindingList,
    /// The `fn {identifier} ( {arguments}` has been parsed so far.
    ParenthesisRight,
    /// The `fn {identifier} ( {arguments} )` has been parsed so far.
    ArrowOrBody,
    /// The `fn {identifier} ( {arguments} ) ->` has been parsed so far.
    ReturnType,
    /// The `fn {identifier} ( {arguments} )` with optional `-> {type}` has been parsed so far.
    Body,
}

impl Default for State {
    fn default() -> Self {
        Self::KeywordFn
    }
}

///
/// The `fn` statement parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The builder of the parsed value.
    builder: FnStatementBuilder,
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses an 'fn' statement.
    ///
    /// '
    /// fn sum(a: u8, b: u8) -> u8 {
    ///     a + b
    /// }
    /// '
    ///
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        mut initial: Option<Token>,
    ) -> Result<(FnStatementBuilder, Option<Token>), Error> {
        loop {
            match self.state {
                State::KeywordFn => {
                    match crate::syntax::parser::take_or_next(initial.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Keyword(Keyword::Fn),
                            location,
                        } => {
                            self.builder.set_location(location);
                            self.state = State::Identifier;
                        }
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::expected_one_of(
                                location,
                                vec!["fn"],
                                lexeme,
                                None,
                            )));
                        }
                    }
                }
                State::Identifier => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Identifier(identifier),
                            location,
                        } => {
                            let identifier = Identifier::new(location, identifier.inner);
                            self.builder.set_identifier(identifier);
                            self.state = State::ParenthesisLeft;
                        }
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::expected_identifier(
                                location,
                                lexeme,
                                Some(HINT_EXPECTED_IDENTIFIER),
                            )));
                        }
                    }
                }
                State::ParenthesisLeft => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                            ..
                        } => self.state = State::ArgumentBindingList,
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::expected_one_of(
                                location,
                                vec!["("],
                                lexeme,
                                Some(HINT_EXPECTED_ARGUMENT_LIST),
                            )));
                        }
                    }
                }
                State::ArgumentBindingList => {
                    let (argument_bindings, next) =
                        BindingPatternListParser::default().parse(stream.clone(), None)?;
                    self.builder.set_argument_bindings(argument_bindings);
                    self.next = next;
                    self.state = State::ParenthesisRight;
                }
                State::ParenthesisRight => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                            ..
                        } => self.state = State::ArrowOrBody,
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::expected_one_of(
                                location,
                                vec![",", ")"],
                                lexeme,
                                None,
                            )));
                        }
                    }
                }
                State::ArrowOrBody => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::MinusGreater),
                            ..
                        } => self.state = State::ReturnType,
                        token => {
                            self.next = Some(token);
                            self.state = State::Body;
                        }
                    }
                }
                State::ReturnType => {
                    let (r#type, next) = TypeParser::default().parse(stream.clone(), None)?;
                    self.next = next;
                    self.builder.set_return_type(r#type);
                    self.state = State::Body;
                }
                State::Body => {
                    let (expression, next) =
                        BlockExpressionParser::default().parse(stream, self.next.take())?;

                    self.builder.set_body(expression);
                    return Ok((self.builder, next));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::error::Error;
    use crate::lexical::stream::TokenStream;
    use crate::lexical::token::lexeme::symbol::Symbol;
    use crate::lexical::token::lexeme::Lexeme;
    use crate::lexical::token::location::Location;
    use crate::syntax::error::Error as SyntaxError;
    use crate::syntax::tree::expression::block::Expression as BlockExpression;
    use crate::syntax::tree::identifier::Identifier;
    use crate::syntax::tree::pattern_binding::variant::Variant as BindingPatternVariant;
    use crate::syntax::tree::pattern_binding::Pattern as BindingPattern;
    use crate::syntax::tree::r#type::variant::Variant as TypeVariant;
    use crate::syntax::tree::r#type::Type;
    use crate::syntax::tree::statement::r#fn::Statement as FnStatement;

    #[test]
    fn ok_returns_unit() {
        let input = r#"fn f(a: field) {}"#;

        let expected = Ok((
            FnStatement::new(
                Location::test(1, 1),
                false,
                false,
                Identifier::new(Location::test(1, 4), "f".to_owned()),
                vec![BindingPattern::new(
                    Location::test(1, 6),
                    BindingPatternVariant::new_binding(
                        Identifier::new(Location::test(1, 6), "a".to_owned()),
                        false,
                    ),
                    Type::new(Location::test(1, 9), TypeVariant::field()),
                )],
                None,
                BlockExpression::new(Location::test(1, 16), vec![], None),
                vec![],
            ),
            None,
        ));

        let result = Parser::default()
            .parse(TokenStream::test(input).wrap(), None)
            .map(|(builder, next)| (builder.finish(), next));

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_returns_type() {
        let input = r#"fn f(a: field) -> field {}"#;

        let expected = Ok((
            FnStatement::new(
                Location::test(1, 1),
                false,
                false,
                Identifier::new(Location::test(1, 4), "f".to_owned()),
                vec![BindingPattern::new(
                    Location::test(1, 6),
                    BindingPatternVariant::new_binding(
                        Identifier::new(Location::test(1, 6), "a".to_owned()),
                        false,
                    ),
                    Type::new(Location::test(1, 9), TypeVariant::field()),
                )],
                Some(Type::new(Location::test(1, 19), TypeVariant::field())),
                BlockExpression::new(Location::test(1, 25), vec![], None),
                vec![],
            ),
            None,
        ));

        let result = Parser::default()
            .parse(TokenStream::test(input).wrap(), None)
            .map(|(builder, next)| (builder.finish(), next));

        assert_eq!(result, expected);
    }

    #[test]
    fn error_expected_identifier() {
        let input = r#"fn (a: u8) -> field {}"#;

        let expected = Err(Error::Syntax(SyntaxError::expected_identifier(
            Location::test(1, 4),
            Lexeme::Symbol(Symbol::ParenthesisLeft),
            Some(super::HINT_EXPECTED_IDENTIFIER),
        )));

        let result = Parser::default()
            .parse(TokenStream::test(input).wrap(), None)
            .map(|(builder, next)| (builder.finish(), next));

        assert_eq!(result, expected);
    }

    #[test]
    fn error_expected_parenthesis_left() {
        let input = r#"fn sort -> field {}"#;

        let expected = Err(Error::Syntax(SyntaxError::expected_one_of(
            Location::test(1, 9),
            vec!["("],
            Lexeme::Symbol(Symbol::MinusGreater),
            Some(super::HINT_EXPECTED_ARGUMENT_LIST),
        )));

        let result = Parser::default()
            .parse(TokenStream::test(input).wrap(), None)
            .map(|(builder, next)| (builder.finish(), next));

        assert_eq!(result, expected);
    }

    #[test]
    fn error_expected_comma_or_parenthesis_right() {
        let input = r#"fn sort(array: [u8; 100]] -> field {}"#;

        let expected = Err(Error::Syntax(SyntaxError::expected_one_of(
            Location::test(1, 25),
            vec![",", ")"],
            Lexeme::Symbol(Symbol::BracketSquareRight),
            None,
        )));

        let result = Parser::default()
            .parse(TokenStream::test(input).wrap(), None)
            .map(|(builder, next)| (builder.finish(), next));

        assert_eq!(result, expected);
    }
}