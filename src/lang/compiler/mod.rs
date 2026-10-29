pub(super) mod iter;
pub(super) mod lexer;
pub(super) mod parser;

mod numeric_literal;

use crate::{lang::compiler::parser::statement::Statement, logger::PolderError};
use lexer::{Lexer, token::TokenType};

pub(crate) type Span = (usize, usize);

#[derive(Debug)]
pub(crate) struct CompilerError {
    #[allow(dead_code)]
    pub(crate) kind: ErrorCode,
    pub(crate) message: String,
    pub(crate) span_message: String,
    pub(crate) span: Span,
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
    UnshiftedUnexpectedToken,
    InvalidExpression,
    StatementNotTerminated,
    InvalidTypeDeclaration,
    WrongArticleForType,
    InvalidNumericLiteral,
    CannotInferVariableType,
    IndefiniteCannotHaveValue,
    UnexpectedCharacter,
    ConstDeclarationMustHaveValue,
    InvalidIdentifier,
    ParseIntError,
    InvalidThenStatement,
    InvalidStatement,
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

pub(crate) fn compile(input: &str) -> Result<Vec<Statement>, CompilerError> {
    parser::Parser::parse(Lexer::lex(input)?)
}
