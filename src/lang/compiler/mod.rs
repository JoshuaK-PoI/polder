pub(crate) mod expression;
pub(crate) mod intrinsic;
mod lexer;
pub(crate) mod parser;
pub(crate) mod statement;
mod token;
pub(crate) mod types;
pub(crate) mod unshift;
mod numeric_literal;

use std::ops::Range;

use crate::logger::ErrorLogger;
use log::trace;
use parser::AST;

use crate::lang::compiler::{
    parser::Parser,
    token::{Token, TokenType},
};

pub(crate) type CompilerResult<T> = Result<T, CompilerError>;

#[derive(Debug)]
pub(crate) struct CompilerError {
    pub(crate) error_code: ErrorCode,
    pub(crate) error_message: String,
    pub(crate) span_message: String,
    pub(crate) span: Range<usize>,
    pub(crate) help: Option<String>,
    pub(crate) info: Option<String>,
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "
            Compiler error:
            - Error code: {}
            - Message: {},
            - Span message: {},
            - Span: {:#?},
            - Help: {:?},
            - Info: {:?}
        ", self.error_code, self.error_message, self.span_message, self.span, self.help, self.info)
    }
}

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
    VariableAlreadyDefined,
    UndefinedVariable,
    InvalidNumericLiteral,
    NoArgumentsProvided,
    CannotInferVariableType,
    IndefiniteCannotHaveValue,
    AssignUndefinedVariable,
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
    lexer: lexer::Lexer<'a>,
    error_logger: ErrorLogger<'a>,
}

impl<'a> Compiler<'a> {
    pub(crate) fn new(input: &'a str, filename: &'a str) -> Compiler<'a> {
        let error_logger = ErrorLogger::new(input, filename);
        Compiler {
            input,
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
            return Err(CompilerError {
                error_code: ErrorCode::ParseError,
                error_message: format!("Cannot parse program due to syntax errors"),
                span: 0..0,
                span_message: format!(""),
                info: Some(format!("Fix syntax errors before continuing")),
                help: None
            });
        }

        if log::max_level() >= log::LevelFilter::Trace {
            trace!("{:^16}|{:^16} @ {}", "Type", "Literal", "Position");
            trace!("{:-<1$}", "", 40);
            tokens.iter().for_each(|token| trace!("{}", token));
        }

        Parser::parse(&mut tokens.iter_mut().peekable())
    }
}
