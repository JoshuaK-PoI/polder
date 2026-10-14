use serde::Serialize;

use super::super::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum Intrinsics {
    Print,
    Write,
    Readline,
}

impl std::fmt::Display for Intrinsics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Intrinsics::Print => "Druk ... af",
                Intrinsics::Write => "Schrijf ... op",
                Intrinsics::Readline => "Lees ... in",
            }
        )
    }
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
