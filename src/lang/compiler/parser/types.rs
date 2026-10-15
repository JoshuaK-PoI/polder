use serde::Serialize;

use super::super::{
    CompilerError, ErrorCode,
    iter::unshift::UnshiftExpect,
    lexer::token::{ARTICLES, Token, TokenStream, TokenType},
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) enum PolderType {
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
        format!(
            "{}",
            match value {
                PolderType::Boolean => "waarheid",
                PolderType::String => "tekst",
                PolderType::Integer => "getal",
                PolderType::MaybeInteger => "getal",
                PolderType::Float => "kommagetal",
                PolderType::Char => "karakter",
            }
        )
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
                    kind: ErrorCode::WrongArticleForType,
                    message: format!("Lekker genederlandst"),
                    span_message: format!("Dat is geen correct Nederlands"),
                    span: article.span.clone(),
                })
            } else {
                Ok(PolderType::from(typedecl.value))
            }
        }
        "waarheid" | "tekst" | "letter" => {
            if ![TokenType::KWArticleDefD, TokenType::KWArticleIndef].contains(&article.type_) {
                Err(CompilerError {
                    kind: ErrorCode::WrongArticleForType,
                    message: format!("Lekker genederlandst"),
                    span_message: format!("Dat is geen correct Nederlands"),
                    span: article.span.clone(),
                })
            } else {
                Ok(PolderType::from(typedecl.value))
            }
        }
        _ => Err(CompilerError {
            kind: ErrorCode::InvalidTypeDeclaration,
            message: format!("'{}' is geen type", typedecl.value),
            span: typedecl.span.clone(),
            span_message: format!("'{}' is geen bekend type ", typedecl.value),
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
                kind: ErrorCode::CannotInferVariableType,
                message: format!("'{}' is geen type", token.type_),
                span_message: format!("'{}' is geen bekend type.", token.value),
                span: token.span.clone(),
            }),
        }
    }

    pub(crate) fn article_for(type_: &Self) -> TokenType {
        match type_ {
            PolderType::Boolean | PolderType::Char | PolderType::String => TokenType::KWArticleDefD,
            PolderType::Integer | PolderType::MaybeInteger | PolderType::Float => {
                TokenType::KWArticleDefH
            }
        }
    }
}
