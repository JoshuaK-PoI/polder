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
use serde::Serialize;

pub(crate) type Span = (usize, usize);

#[derive(Debug)]
pub(crate) struct CompilerError {
    #[allow(dead_code)]
    pub(crate) kind: ErrorCode,
    pub(crate) message: String,
    pub(crate) span_message: Option<String>,
    pub(crate) span: Span,
    pub(crate) help: Option<Help>,
    pub(crate) infos: Vec<Info>,
}

impl CompilerError {
    pub(crate) fn new(kind: ErrorCode, message: String, span: Span) -> Self {
        CompilerError {
            kind,
            message,
            span,
            span_message: None,
            help: None,
            infos: Vec::new(),
        }
    }

    pub(crate) fn with_span_message(mut self, span_message: String) -> Self {
        self.span_message = Some(span_message);
        self
    }

    pub(crate) fn with_help(mut self, message: String) -> Self {
        self.help = Some(message);
        self
    }

    pub(crate) fn with_info(mut self, span: Span, message: String) -> Self {
        self.infos.push(Info { span, message });
        self
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Info {
    span: Span,
    message: String,
}
pub(crate) type Help = String;

impl Info {
    pub(crate) fn span(&self) -> Span {
        self.span
    }
    pub(crate) fn message(&self) -> String {
        self.message.clone()
    }
}

impl From<CompilerError> for PolderError {
    fn from(value: CompilerError) -> Self {
        PolderError {
            message: value.message.clone(),
            span_message: value.span_message.unwrap_or(value.message),
            span: value.span,
            help_message: value.help,
            infos: value.infos,
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
    UndefinedVariable,
    MismatchedType,
    UninitializedVariable,
    ConstVariable,
    DefiniteMustHaveValue,
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
