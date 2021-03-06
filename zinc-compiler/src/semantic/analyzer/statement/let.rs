//!
//! The `let` statement semantic analyzer.
//!

use std::cell::RefCell;
use std::rc::Rc;

use zinc_syntax::LetStatement;

use crate::generator::statement::r#let::Statement as GeneratorDeclarationStatement;
use crate::semantic::analyzer::expression::Analyzer as ExpressionAnalyzer;
use crate::semantic::analyzer::rule::Rule as TranslationRule;
use crate::semantic::binding::Binder;
use crate::semantic::element::error::Error as ElementError;
use crate::semantic::element::r#type::error::Error as TypeError;
use crate::semantic::element::r#type::Type;
use crate::semantic::element::Element;
use crate::semantic::error::Error;
use crate::semantic::scope::memory_type::MemoryType;
use crate::semantic::scope::Scope;

///
/// The `let` statement semantic analyzer.
///
pub struct Analyzer {}

impl Analyzer {
    ///
    /// Defines a variable and returns its IR for the next compiler phase.
    ///
    pub fn define(
        scope: Rc<RefCell<Scope>>,
        statement: LetStatement,
    ) -> Result<Option<GeneratorDeclarationStatement>, Error> {
        let (element, expression) = ExpressionAnalyzer::new(scope.clone(), TranslationRule::Value)
            .analyze(statement.expression)?;

        let r#type = if let Some(r#type) = statement.binding.r#type {
            let r#type = Type::try_from_syntax(r#type, scope.clone())?;
            element
                .cast(Element::Type(r#type.clone()))
                .map_err(Error::Element)?;
            r#type
        } else {
            Type::from_element(&element, scope.clone())?
        };

        if !r#type.is_instantiatable(false) {
            return Err(Error::Element(ElementError::Type(
                TypeError::InstantiationForbidden {
                    location: statement.binding.location,
                    found: r#type.to_string(),
                },
            )));
        }

        let memory_type = match r#type {
            Type::Contract(_) => MemoryType::ContractInstance,
            _ => MemoryType::Stack,
        };

        let bindings =
            Binder::bind_variables(statement.binding.pattern, r#type, memory_type, scope)?;
        Ok(if bindings.is_empty() {
            None
        } else {
            Some(GeneratorDeclarationStatement::new(
                statement.location,
                bindings,
                expression,
            ))
        })
    }
}
