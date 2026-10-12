use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, ErrorCode,
    token::{TokenStream, TokenType},
    unshift::UnshiftExpect,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) enum PolderType {
    Uninitialized,
    Boolean,
    String,
    Integer,
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

pub(crate) fn parse_type(tokens: &mut TokenStream) -> Result<Option<PolderType>, CompilerError> {
    let article = tokens.unshift_expect_any(&[
        TokenType::KWArticleDefD,
        TokenType::KWArticleDefH,
        TokenType::KWArticleIndef,
    ])?;

    let typedecl = tokens.unshift_expect(TokenType::Identifier)?;

    match typedecl.value.as_str() {
        "getal" | "kommagetal" => {
            if article.type_ != TokenType::KWArticleDefH {
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
                Ok(Some(PolderType::from(typedecl.value)))
            }
        }
        "waarheid" | "tekst" | "letter" => {
            if article.type_ != TokenType::KWArticleDefD {
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
                Ok(Some(PolderType::from(typedecl.value)))
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
