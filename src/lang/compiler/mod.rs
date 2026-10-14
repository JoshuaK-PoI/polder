pub(super) mod iter;
pub(super) mod parser;
pub(super) mod lexer;

mod numeric_literal;

use std::ops::Range;

use crate::logger::{ErrorLogger, PolderError};
use log::trace;
use parser::AST;

use lexer::{
    Lexer,
    token::{Token, TokenType},
};

#[derive(Debug)]
pub(crate) struct CompilerError {
    pub(crate) kind: ErrorCode,
    pub(crate) message: String,
    pub(crate) span_message: String,
    pub(crate) span: Range<usize>,
}

impl From<CompilerError> for PolderError {
    fn from(value: CompilerError) -> Self {
        PolderError {
            message: value.message,
            span_message: value.span_message,
            span: value.span,
        }
    }
}

pub(crate) type CompilerResult<T> = Result<T, CompilerError>;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ErrorCode {
    NoTokensLeft,
    UnexpectedToken,
    UnknownToken,
    UnshiftedUnexpectedToken,
    InvalidExpression,
    StatementNotTerminated,
    InvalidTypeDeclaration,
    WrongArticleForType,
    ParseError,
    InvalidNumericLiteral,
    CannotInferVariableType,
    IndefiniteCannotHaveValue,
}

impl From<ErrorCode> for String {
    fn from(token: ErrorCode) -> String {
        format!("{:?}", token)
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}
pub(crate) fn compile(input: &str, filename: &str) -> Result<AST, CompilerError> {
    let error_logger = ErrorLogger::new(input, filename);
    let mut tokens = Lexer::new(input).lex();

    let unknown_tokens: Vec<&Token> = tokens
        .iter()
        .filter(|token| token.type_ == TokenType::Unknown)
        .collect();

    if unknown_tokens.len() > 0 {
        let errors: Vec<PolderError> = unknown_tokens
            .iter()
            .map(|token| CompilerError {
                kind: ErrorCode::UnknownToken,
                message: format!("Unknown token: {:?}", token.type_),
                span: (**token).span.clone(),
                span_message: format!("This token is unknown to the compiler"),
            })
            .map(|e| e.into())
            .collect();
        error_logger.report_many(&errors);
        return Err(CompilerError {
            kind: ErrorCode::ParseError,
            message: format!("Cannot parse program due to syntax errors"),
            span: 0..0,
            span_message: format!(""),
        });
    }

    if log::max_level() >= log::LevelFilter::Trace {
        trace!("{:^16}|{:^16} @ {}", "Type", "Literal", "Position");
        trace!("{:-<1$}", "", 40);
        tokens.iter().for_each(|token| trace!("{}", token));
    }

    parser::parse(&mut tokens.iter_mut().peekable())
}
