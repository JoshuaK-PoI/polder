pub(super) mod iter;
pub(super) mod lexer;
pub(super) mod parser;

mod numeric_literal;

use crate::{
    lang::{
        compiler::parser::{Parser, statement::Statement},
        typechecker::Typechecker,
    },
    logger::PolderError,
};
use lexer::{Lexer, token::TokenType};

pub(crate) type Span = (usize, usize);

#[derive(Debug)]
pub(crate) struct CompilerError {
    #[allow(dead_code)]
    pub(crate) kind: ErrorCode,
    pub(crate) message: String,
    pub(crate) span_message: Option<String>,
    pub(crate) span: Span,
    pub(crate) help: Option<Help>,
}

impl CompilerError {
    pub(crate) fn new(kind: ErrorCode, message: String, span: Span) -> Self {
        CompilerError {
            kind,
            message,
            span,
            span_message: None,
            help: None,
        }
    }

    pub(crate) fn with_span_message(mut self, span_message: String) -> Self {
        self.span_message = Some(span_message);
        self
    }

    pub(crate) fn with_help(mut self, span: Span, message: String) -> Self {
        self.help = Some(Help { span, message });
        self
    }
}

#[derive(Debug)]
pub(crate) struct Help {
    span: Span,
    message: String,
}

impl Help {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        self.message.clone()
    }
}

impl From<CompilerError> for PolderError {
    fn from(value: CompilerError) -> Self {
        PolderError {
            message: value.message.clone(),
            span_message: value.span_message.unwrap_or(value.message),
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
    RedefinitionOfVariable,
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
    let tokens = Lexer::lex(input)?;
    let ast = Parser::parse(tokens)?;
    Typechecker::typecheck(&ast)?;

    Ok(ast)
}
