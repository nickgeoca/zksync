//!
//! The mod statement parser.
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
use crate::syntax::Identifier;
use crate::syntax::ModStatement;
use crate::syntax::ModStatementBuilder;

#[derive(Default)]
pub struct Parser {
    builder: ModStatementBuilder,
    next: Option<Token>,
}

impl Parser {
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        mut initial: Option<Token>,
    ) -> Result<(ModStatement, Option<Token>), Error> {
        match crate::syntax::take_or_next(initial.take(), stream.clone())? {
            Token {
                lexeme: Lexeme::Keyword(Keyword::Mod),
                location,
            } => {
                self.builder.set_location(location);
            }
            Token { lexeme, location } => {
                return Err(Error::Syntax(SyntaxError::expected_one_of(
                    location,
                    vec!["mod"],
                    lexeme,
                )));
            }
        }

        match crate::syntax::take_or_next(self.next.take(), stream.clone())? {
            Token {
                lexeme: Lexeme::Identifier(identifier),
                location,
            } => {
                let identifier = Identifier::new(location, identifier.name);
                self.builder.set_identifier(identifier);
            }
            Token { lexeme, location } => {
                return Err(Error::Syntax(SyntaxError::expected_identifier(
                    location, lexeme,
                )))
            }
        }

        match crate::syntax::take_or_next(self.next.take(), stream)? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::Semicolon),
                ..
            } => Ok((self.builder.finish(), None)),
            Token { lexeme, location } => Err(Error::Syntax(SyntaxError::expected_one_of(
                location,
                vec![";"],
                lexeme,
            ))),
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
    use crate::syntax::ModStatement;

    #[test]
    fn ok() {
        let input = r#"mod jabberwocky;"#;

        let expected = Ok((
            ModStatement::new(
                Location::new(1, 1),
                Identifier::new(Location::new(1, 5), "jabberwocky".to_owned()),
            ),
            None,
        ));

        let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);

        assert_eq!(result, expected);
    }
}
