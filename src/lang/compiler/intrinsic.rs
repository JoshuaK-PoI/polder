use serde::Serialize;

use crate::lang::compiler::token::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum Intrinsics {
    Print,
    Write,
    Readline
}

impl Intrinsics {
    pub(crate) fn expected_start_token(&self) -> TokenType {
        match self {
            Intrinsics::Print => TokenType::KWDruk,
            Intrinsics::Write => TokenType::KWSchrijf,
            Intrinsics::Readline => TokenType::KWLees,
        }
    }

    pub(crate) fn expected_end_token(&self) -> TokenType {
        match self {
            Intrinsics::Print => TokenType::KWAf,
            Intrinsics::Write => TokenType::KWOp,
            Intrinsics::Readline => TokenType::KWIn,
        }
    }
}
