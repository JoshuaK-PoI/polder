use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, ErrorCode,
    token::{ARTICLES, Token, TokenStream, TokenType},
    unshift::UnshiftExpect,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) enum PolderType {
    Uninitialized,
    Boolean,
    String,
    Integer,
    MaybeInteger,
    Float,
    Char,
}

impl From<String> for PolderType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "getal" => PolderType::Integer,
            "kommagetal" => PolderType::Float,
            "waarheid" => PolderType::Boolean,
            "tekst" => PolderType::String,
            "letter" => PolderType::Char,
            _ => panic!("Don't know about type {}", value),
        }
    }
}

impl From<&PolderType> for String {
    fn from(value: &PolderType) -> Self {
        format!("{:?}", value)
    }
}

impl std::fmt::Display for PolderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

pub(crate) fn parse_type(tokens: &mut TokenStream) -> Result<PolderType, CompilerError> {
    if !tokens.next_matches_any(ARTICLES) {
        return PolderType::infer(tokens.peek().unwrap());
    }

    let article = tokens.unshift_expect_any(&[
        TokenType::KWArticleDefD,
        TokenType::KWArticleDefH,
        TokenType::KWArticleIndef,
    ])?;

    let typedecl = tokens.unshift_expect(TokenType::Identifier)?;

    match typedecl.value.as_str() {
        "getal" | "kommagetal" => {
            if ![TokenType::KWArticleDefH, TokenType::KWArticleIndef].contains(&article.type_) {
                Err(CompilerError {
                    error_code: ErrorCode::WrongArticleForType,
                    error_message: format!("Lekker genederlandst"),
                    span_message: format!("Dat is geen correct Nederlands"),
                    span: article.span.clone(),
                    help: Some(format!(
                        "Bedoelde je misschien {} {}?",
                        "het", typedecl.value
                    )),
                    info: None,
                })
            } else {
                Ok(PolderType::from(typedecl.value))
            }
        }
        "waarheid" | "tekst" | "letter" => {
            if ![TokenType::KWArticleDefD, TokenType::KWArticleIndef].contains(&article.type_) {
                Err(CompilerError {
                    error_code: ErrorCode::WrongArticleForType,
                    error_message: format!("Lekker genederlandst"),
                    span_message: format!("Dat is geen correct Nederlands"),
                    span: article.span.clone(),
                    help: Some(format!(
                        "Bedoelde je misschien {} {}?",
                        "de", typedecl.value
                    )),
                    info: None,
                })
            } else {
                Ok(PolderType::from(typedecl.value))
            }
        }
        _ => Err(CompilerError {
            error_code: ErrorCode::InvalidTypeDeclaration,
            error_message: format!("Unknown type: {}", typedecl.value),
            span: typedecl.span.clone(),
            span_message: format!("{} is not a known type", typedecl.value),
            help: Some(format!(
                "Supported types are: {:?}",
                &[
                    "het getal",
                    "het kommagetal",
                    "de waarheid",
                    "de tekst",
                    "de letter",
                ]
            )),
            info: None,
        }),
    }
}

impl PolderType {
    pub(crate) fn infer(token: &Token) -> Result<Self, CompilerError> {
        match token.type_ {
            TokenType::TrueLiteral => Ok(PolderType::Boolean),
            TokenType::FalseLiteral => Ok(PolderType::Boolean),
            TokenType::StringLiteral => Ok(PolderType::String),
            TokenType::IntegerLiteral => Ok(PolderType::Integer),
            TokenType::FloatLiteral => Ok(PolderType::Float),
            TokenType::CharLiteral => Ok(PolderType::Char),
            TokenType::Identifier => Ok(PolderType::MaybeInteger),
            _ => Err(CompilerError {
                error_code: ErrorCode::CannotInferVariableType,
                error_message: format!("Cannot infer variable type from {}", token.type_),
                span_message: format!("'{}' has no associated variable type", token.value),
                span: token.span.clone(),
                help: None,
                info: None,
            }),
        }
    }
}
