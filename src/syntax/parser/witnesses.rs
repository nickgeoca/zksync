//!
//! The witnesses parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::lexical::Keyword;
use crate::lexical::Lexeme;
use crate::lexical::Symbol;
use crate::lexical::Token;
use crate::lexical::TokenStream;
use crate::syntax::Error as SyntaxError;
use crate::syntax::Identifier;
use crate::syntax::TypeParser;
use crate::syntax::Witness;
use crate::syntax::WitnessBuilder;
use crate::Error;

#[derive(Debug, Clone, Copy)]
pub enum State {
    KeywordWitness,
    BracketOpen,
    ElementIdentifierOrBracketClose,
    ElementColon,
    ElementType,
    ElementSemicolon,
}

impl Default for State {
    fn default() -> Self {
        State::KeywordWitness
    }
}

#[derive(Default)]
pub struct Parser {
    state: State,
    witnesses: Vec<Witness>,
    builder: WitnessBuilder,
}

impl Parser {
    pub fn parse(mut self, stream: Rc<RefCell<TokenStream>>) -> Result<Vec<Witness>, Error> {
        match stream.borrow_mut().peek() {
            Some(Ok(Token {
                lexeme: Lexeme::Keyword(Keyword::Witness),
                ..
            })) => {}
            _ => return Ok(self.witnesses),
        }

        loop {
            match self.state {
                State::KeywordWitness => match stream.borrow_mut().next() {
                    Some(Ok(Token {
                        lexeme: Lexeme::Keyword(Keyword::Witness),
                        ..
                    })) => self.state = State::BracketOpen,
                    Some(Ok(Token { lexeme, location })) => {
                        return Err(Error::Syntax(SyntaxError::Expected(
                            location,
                            ["witness"].to_vec(),
                            lexeme,
                        )));
                    }
                    Some(Err(error)) => return Err(Error::Lexical(error)),
                    None => return Err(Error::Syntax(SyntaxError::UnexpectedEnd)),
                },
                State::BracketOpen => match stream.borrow_mut().next() {
                    Some(Ok(Token {
                        lexeme: Lexeme::Symbol(Symbol::BracketCurlyLeft),
                        ..
                    })) => self.state = State::ElementIdentifierOrBracketClose,
                    Some(Ok(Token { lexeme, location })) => {
                        return Err(Error::Syntax(SyntaxError::Expected(
                            location,
                            ["{"].to_vec(),
                            lexeme,
                        )));
                    }
                    Some(Err(error)) => return Err(Error::Lexical(error)),
                    None => return Err(Error::Syntax(SyntaxError::UnexpectedEnd)),
                },
                State::ElementIdentifierOrBracketClose => match stream.borrow_mut().next() {
                    Some(Ok(Token {
                        lexeme: Lexeme::Symbol(Symbol::BracketCurlyRight),
                        ..
                    })) => return Ok(self.witnesses),
                    Some(Ok(Token {
                        lexeme: Lexeme::Identifier(identifier),
                        location,
                    })) => {
                        let identifier = Identifier::new(location, identifier.name().to_owned());
                        self.builder.set_identifier(identifier);
                        self.state = State::ElementColon;
                    }
                    Some(Ok(Token { lexeme, location })) => {
                        return Err(Error::Syntax(SyntaxError::Expected(
                            location,
                            ["{identifier}", "}"].to_vec(),
                            lexeme,
                        )));
                    }
                    Some(Err(error)) => return Err(Error::Lexical(error)),
                    None => return Err(Error::Syntax(SyntaxError::UnexpectedEnd)),
                },
                State::ElementColon => match stream.borrow_mut().next() {
                    Some(Ok(Token {
                        lexeme: Lexeme::Symbol(Symbol::Colon),
                        ..
                    })) => self.state = State::ElementType,
                    Some(Ok(Token { lexeme, location })) => {
                        return Err(Error::Syntax(SyntaxError::Expected(
                            location,
                            [":"].to_vec(),
                            lexeme,
                        )));
                    }
                    Some(Err(error)) => return Err(Error::Lexical(error)),
                    None => return Err(Error::Syntax(SyntaxError::UnexpectedEnd)),
                },
                State::ElementType => {
                    let r#type = TypeParser::default().parse(stream.clone())?;
                    self.builder.set_type(r#type);
                    self.state = State::ElementSemicolon;
                }
                State::ElementSemicolon => match stream.borrow_mut().next() {
                    Some(Ok(Token {
                        lexeme: Lexeme::Symbol(Symbol::Semicolon),
                        ..
                    })) => {
                        let witness = self.builder.build();
                        log::trace!("Witness: {:?}", witness);
                        self.witnesses.push(witness);
                        self.state = State::ElementIdentifierOrBracketClose;
                    }
                    Some(Ok(Token { lexeme, location })) => {
                        return Err(Error::Syntax(SyntaxError::Expected(
                            location,
                            [";"].to_vec(),
                            lexeme,
                        )));
                    }
                    Some(Err(error)) => return Err(Error::Lexical(error)),
                    None => return Err(Error::Syntax(SyntaxError::UnexpectedEnd)),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::Parser;
    use crate::lexical::Location;
    use crate::lexical::TokenStream;
    use crate::syntax::Identifier;
    use crate::syntax::Type;
    use crate::syntax::TypeVariant;
    use crate::syntax::Witness;

    #[test]
    fn ok() {
        let code = r#"
    witness {
        a: uint232;
    }
"#;

        let expected = vec![Witness::new(
            Identifier::new(Location::new(3, 9), "a".to_owned()),
            Type::new(Location::new(3, 12), TypeVariant::Uint { bitlength: 232 }),
        )];

        let result = Parser::default()
            .parse(Rc::new(RefCell::new(TokenStream::new(code.to_owned()))))
            .expect("Syntax error");

        assert_eq!(expected, result);
    }
}
