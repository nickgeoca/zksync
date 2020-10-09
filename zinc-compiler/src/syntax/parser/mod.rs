//!
//! The syntax parser.
//!

pub mod attribute;
pub mod expression;
pub mod field;
pub mod field_list;
pub mod pattern_binding;
pub mod pattern_binding_list;
pub mod pattern_match;
pub mod statement;
pub mod r#type;
pub mod variant;
pub mod variant_list;

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::lexical::stream::TokenStream;
use crate::lexical::token::lexeme::Lexeme;
use crate::lexical::token::Token;
use crate::syntax::parser::statement::local_mod::Parser as ModuleLocalStatementParser;
use crate::syntax::tree::module::Module;

///
/// The module top-level parser.
///
#[derive(Default)]
pub struct Parser {
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses a list of module level statements.
    ///
    pub fn parse(mut self, input: &str, file: usize) -> Result<Module, Error> {
        let stream = TokenStream::new(input, file).wrap();

        let mut statements = Vec::new();
        loop {
            match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                Token {
                    lexeme: Lexeme::Eof,
                    ..
                } => break,
                token => {
                    let (statement, next) =
                        ModuleLocalStatementParser::default().parse(stream.clone(), Some(token))?;
                    self.next = next;
                    statements.push(statement);
                }
            }
        }

        Ok(Module::new(statements))
    }
}

///
/// Returns the `token` value if it is `Some(_)`, otherwise takes the next token from the `stream`.
///
pub fn take_or_next(
    mut token: Option<Token>,
    stream: Rc<RefCell<TokenStream>>,
) -> Result<Token, Error> {
    match token.take() {
        Some(token) => Ok(token),
        None => Ok(stream.borrow_mut().next()?),
    }
}