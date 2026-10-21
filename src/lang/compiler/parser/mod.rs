pub(crate) mod expression;
pub(crate) mod intrinsic;
pub(crate) mod statement;
pub(crate) mod types;

use crate::lang::compiler::{iter::unshift::{Unshift, UnshiftExpect}, lexer::token::TokenType};

use super::{CompilerResult, lexer::token::TokenStream};
use statement::{Statement, parse as stmt_parse};

pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<Vec<Statement>> {
    let mut ast = Vec::new();

    while tokens.peek().is_some() {
        if tokens.next_matches(TokenType::Comment) {
            tokens.unshift();
            continue;
        }
        ast.push(stmt_parse(tokens, false)?);
    }

    Ok(ast)
}
