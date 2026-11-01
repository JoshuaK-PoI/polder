mod expression;
mod intrinsic;
mod statement;

use std::collections::HashMap;

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode, Span,
    parser::{statement::Statement, types::PolderType},
};

pub(crate) struct Var {
    loc: Span,
    type_: PolderType,
    is_const: bool,
    initialized: bool,
    modified: bool,
}

pub struct Typechecker {
    scopes: Vec<HashMap<String, Var>>,
}

impl Typechecker {
    fn declare(
        &mut self,
        identifier: String,
        span: Span,
        type_: PolderType,
        is_const: bool,
        initialized: bool,
    ) -> CompilerResult<()> {
        let scope = self
            .scopes
            .last_mut()
            .expect("Expected at least one scope; something destroyed the global scope");
        if let Some(var) = scope.get(&identifier) {
            Err(CompilerError::new(
                ErrorCode::RedefinitionOfVariable,
                format!("{} bestaat al", identifier),
                span,
            )
            .with_info(var.loc, format!("{} hier gedefinieerd", identifier)))
        } else {
            scope.insert(
                identifier,
                Var {
                    loc: span,
                    type_,
                    is_const,
                    initialized,
                    modified: false,
                },
            );
            Ok(())
        }
    }

    fn find_identifier(&self, identifier: &str) -> Option<&Var> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(identifier) {
                return Some(var);
            }
        }

        None
    }

    fn find_identifier_mut(&mut self, identifier: &str) -> Option<&mut Var> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get_mut(identifier) {
                return Some(var);
            }
        }

        None
    }
}

trait Typecheck {
    type Value;

    fn typecheck(&self, typechecker: &mut Typechecker) -> CompilerResult<Self::Value>;
}

impl Typechecker {
    fn new() -> Self {
        Typechecker {
            scopes: vec![HashMap::new()],
        }
    }
    pub(crate) fn typecheck(ast: &Vec<Statement>) -> CompilerResult<()> {
        let mut typechecker = Self::new();

        for statement in ast {
            statement.typecheck(&mut typechecker)?;
        }

        Ok(())
    }
}
