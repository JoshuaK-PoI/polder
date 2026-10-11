use serde::Serialize;

use crate::lang::compiler::{
    CompilerResult,
    statement::{SpannedStatement, Statement},
    token::TokenStream,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AST {
    statements: Vec<SpannedStatement>,
}

impl AST {
    pub(crate) fn next(&mut self) -> Option<SpannedStatement> {
        self.statements.pop()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Parser {}

impl Parser {
    pub(crate) fn parse<'a>(tokens: &mut TokenStream) -> CompilerResult<AST> {
        let mut ast = AST {
            statements: Vec::new(),
        };

        while let Some(_) = tokens.peek() {
            ast.statements.push(Statement::parse(tokens)?);
        }

        Ok(ast)
    }
}
