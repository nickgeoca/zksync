//!
//! The struct statement parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::lexical::Keyword;
use crate::lexical::Lexeme;
use crate::lexical::Symbol;
use crate::lexical::Token;
use crate::lexical::TokenStream;
use crate::syntax::Error as SyntaxError;
use crate::syntax::FieldListParser;
use crate::syntax::Identifier;
use crate::syntax::StructStatement;
use crate::syntax::StructStatementBuilder;

#[derive(Debug, Clone, Copy)]
pub enum State {
    KeywordStruct,
    Identifier,
    BracketCurlyLeftOrEnd,
    FieldList,
    BracketCurlyRight,
}

impl Default for State {
    fn default() -> Self {
        State::KeywordStruct
    }
}

#[derive(Default)]
pub struct Parser {
    state: State,
    builder: StructStatementBuilder,
    next: Option<Token>,
}

impl Parser {
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        mut initial: Option<Token>,
    ) -> Result<(StructStatement, Option<Token>), Error> {
        loop {
            match self.state {
                State::KeywordStruct => {
                    match crate::syntax::take_or_next(initial.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Keyword(Keyword::Struct),
                            location,
                        } => {
                            self.builder.set_location(location);
                            self.state = State::Identifier;
                        }
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::Expected(
                                location,
                                vec!["struct"],
                                lexeme,
                            )));
                        }
                    }
                }
                State::Identifier => {
                    match crate::syntax::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Identifier(identifier),
                            location,
                        } => {
                            let identifier = Identifier::new(location, identifier.name);
                            self.builder.set_identifier(identifier);
                            self.state = State::BracketCurlyLeftOrEnd;
                        }
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::Expected(
                                location,
                                vec!["{identifier}"],
                                lexeme,
                            )));
                        }
                    }
                }
                State::BracketCurlyLeftOrEnd => {
                    match crate::syntax::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::BracketCurlyLeft),
                            ..
                        } => {
                            self.state = State::FieldList;
                        }
                        token => return Ok((self.builder.finish(), Some(token))),
                    }
                }
                State::FieldList => {
                    let (fields, next) = FieldListParser::default().parse(stream.clone(), None)?;
                    self.builder.set_fields(fields);
                    self.next = next;
                    self.state = State::BracketCurlyRight;
                }
                State::BracketCurlyRight => {
                    return match crate::syntax::take_or_next(self.next.take(), stream)? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::BracketCurlyRight),
                            ..
                        } => Ok((self.builder.finish(), None)),
                        Token { lexeme, location } => Err(Error::Syntax(SyntaxError::Expected(
                            location,
                            vec!["}"],
                            lexeme,
                        ))),
                    };
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
    use crate::lexical::Lexeme;
    use crate::lexical::Location;
    use crate::lexical::Symbol;
    use crate::lexical::Token;
    use crate::lexical::TokenStream;
    use crate::syntax::Field;
    use crate::syntax::Identifier;
    use crate::syntax::StructStatement;
    use crate::syntax::Type;
    use crate::syntax::TypeVariant;

    #[test]
    fn ok_single() {
        let input = r#"
    struct Test {
        a: u232,
    }
"#;

        let expected = Ok((
            StructStatement::new(
                Location::new(2, 5),
                Identifier::new(Location::new(2, 12), "Test".to_owned()),
                vec![Field::new(
                    Location::new(3, 9),
                    Identifier::new(Location::new(3, 9), "a".to_owned()),
                    Type::new(Location::new(3, 12), TypeVariant::integer_unsigned(232)),
                )],
            ),
            None,
        ));

        let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_multiple() {
        let input = r#"
    struct Test {
        a: u232,
        b: u232,
        c: u232,
    }
"#;

        let expected = Ok((
            StructStatement::new(
                Location::new(2, 5),
                Identifier::new(Location::new(2, 12), "Test".to_owned()),
                vec![
                    Field::new(
                        Location::new(3, 9),
                        Identifier::new(Location::new(3, 9), "a".to_owned()),
                        Type::new(Location::new(3, 12), TypeVariant::integer_unsigned(232)),
                    ),
                    Field::new(
                        Location::new(4, 9),
                        Identifier::new(Location::new(4, 9), "b".to_owned()),
                        Type::new(Location::new(4, 12), TypeVariant::integer_unsigned(232)),
                    ),
                    Field::new(
                        Location::new(5, 9),
                        Identifier::new(Location::new(5, 9), "c".to_owned()),
                        Type::new(Location::new(5, 12), TypeVariant::integer_unsigned(232)),
                    ),
                ],
            ),
            None,
        ));

        let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_empty_with_brackets() {
        let input = r#"
    struct Test {}
"#;

        let expected = Ok((
            StructStatement::new(
                Location::new(2, 5),
                Identifier::new(Location::new(2, 12), "Test".to_owned()),
                vec![],
            ),
            None,
        ));

        let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_empty_with_semicolon() {
        let input = r#"
    struct Test;
"#;

        let expected = Ok((
            StructStatement::new(
                Location::new(2, 5),
                Identifier::new(Location::new(2, 12), "Test".to_owned()),
                vec![],
            ),
            Some(Token::new(
                Lexeme::Symbol(Symbol::Semicolon),
                Location::new(2, 16),
            )),
        ));

        let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);

        assert_eq!(result, expected);
    }
}