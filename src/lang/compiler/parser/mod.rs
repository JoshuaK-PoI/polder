pub(crate) mod expression;
pub(crate) mod intrinsic;
pub(crate) mod statement;
pub(crate) mod types;

use super::{CompilerResult, lexer::token::TokenStream};
use statement::{SpannedStatement, Statement};

pub(crate) type AST = Vec<SpannedStatement>;

pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<AST> {
    let mut ast = Vec::new();

    while let Some(_) = tokens.peek() {
        ast.push(Statement::parse(tokens, false)?);
    }

    Ok(ast)
}
