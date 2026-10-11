pub(crate) mod expression;
pub(crate) mod intrinsic;
mod lexer;
pub(crate) mod parser;
pub(crate) mod statement;
mod token;
mod unshift;

use std::ops::Range;

use crate::logger::ErrorLogger;
use log::trace;
use parser::AST;

use crate::lang::compiler::{
    parser::Parser,
    token::{Token, TokenType},
};

pub(crate) type CompilerResult<T> = Result<T, CompilerError>;

pub(crate) struct CompilerError {
    pub(crate) error_code: ErrorCode,
    pub(crate) error_message: String,
    pub(crate) span_message: String,
    pub(crate) span: Range<usize>,
    pub(crate) help: Option<String>,
    pub(crate) info: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ErrorCode {
    NoTokensLeft,
    UnexpectedToken,
    UnknownToken,
    UnshiftedUnexpectedToken,
    InvalidExpression,
    StatementNotTerminated,
    UnknownStatement,
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
pub(crate) struct Compiler<'a> {
    pub(crate) input: &'a str,
    pub(crate) filename: &'a str,
    lexer: lexer::Lexer<'a>,
    error_logger: ErrorLogger<'a>,
}

impl<'a> Compiler<'a> {
    pub(crate) fn new(input: &'a str, filename: &'a str) -> Compiler<'a> {
        let error_logger = ErrorLogger::new(input, filename);
        Compiler {
            input,
            filename,
            lexer: lexer::Lexer::new(),
            error_logger,
        }
    }

    pub(crate) fn compile(&'a mut self) -> Result<AST, CompilerError> {
        let mut tokens = Vec::new();
        self.lexer.lex(self.input, &mut tokens);

        let unknown_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|token| token.type_ == TokenType::Unknown)
            .collect();

        if unknown_tokens.len() > 0 {
            let errors: Vec<CompilerError> = unknown_tokens
                .iter()
                .map(|token| CompilerError {
                    error_code: ErrorCode::UnknownToken,
                    error_message: format!("Unknown token: {:?}", token.type_),
                    span: (**token).span.clone(),
                    span_message: format!("This token is unknown to the compiler"),
                    help: None,
                    info: None,
                })
                .collect();
            self.error_logger.report_many(&errors);
        }

        if log::max_level() >= log::LevelFilter::Trace {
            trace!("{:#04}..{:#04} {}", "Byte", "Rnge", "Token");

            trace!("{:-<1$}", "", 40);
            tokens.iter().for_each(|token| trace!("{}", token));
        }

        let ast = Parser::parse(&mut tokens.iter_mut().peekable())?;

        trace!("{:?}", ast);

        Ok(ast)
    }
}
