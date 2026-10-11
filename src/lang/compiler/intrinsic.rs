use serde::Serialize;

use crate::lang::compiler::token::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum Intrinsics {
    Print,
}

impl Intrinsics {
    pub(crate) fn expected_start_token(&self) -> TokenType {
        match self {
            Intrinsics::Print => TokenType::Druk,
        }
    }

    pub(crate) fn expected_end_token(&self) -> TokenType {
        match self {
            Intrinsics::Print => TokenType::Af,
        }
    }
}
