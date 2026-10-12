use crate::lang::compiler::{
    CompilerResult,
    statement::{SpannedStatement, Statement},
    token::TokenStream
};

pub(crate) type AST = Vec<SpannedStatement>;

#[derive(Debug, Clone)]
pub(crate) struct Parser {}

impl Parser {
    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<AST> {
        let mut ast = Vec::new();

        while let Some(_) = tokens.peek() {
            ast.push(Statement::parse(tokens)?);
        }

        Ok(ast)
    }
}
