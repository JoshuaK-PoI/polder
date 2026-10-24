use serde::Serialize;

use crate::lang::compiler::{lexer::token::Token, parser::Parser};

use super::super::{CompilerError, ErrorCode, lexer::token::TokenType};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(crate) enum PolderType {
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
        format!(
            "{}",
            match value {
                PolderType::Boolean => "waarheid",
                PolderType::String => "tekst",
                PolderType::Integer => "getal",
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

impl Parser {
    pub(crate) fn parse_type(&mut self) -> Result<(PolderType, (usize, usize)), CompilerError> {
        let article = self.expect_any_of(&[
            TokenType::KWArticleDefD,
            TokenType::KWArticleDefH,
            TokenType::KWArticleIndef,
        ])?;

        let article_type = article.type_;
        let article_span = article.span;

        let typedecl = self.expect(TokenType::Identifier)?;

        match typedecl.value.as_str() {
            "getal" | "kommagetal" => {
                if ![TokenType::KWArticleDefH, TokenType::KWArticleIndef].contains(&article_type) {
                    Err(CompilerError {
                        kind: ErrorCode::WrongArticleForType,
                        message: format!("Lekker genederlandst"),
                        span_message: format!("Dat is geen correct Nederlands"),
                        span: article_span,
                    })
                } else {
                    Ok((
                        PolderType::from(typedecl.value.clone()),
                        (article_span.0, typedecl.span.1),
                    ))
                }
            }
            "waarheid" | "tekst" | "letter" => {
                if ![TokenType::KWArticleDefD, TokenType::KWArticleIndef].contains(&article_type) {
                    Err(CompilerError {
                        kind: ErrorCode::WrongArticleForType,
                        message: format!("Lekker genederlandst"),
                        span_message: format!("Dat is geen correct Nederlands"),
                        span: article_span,
                    })
                } else {
                    Ok((
                        PolderType::from(typedecl.value.clone()),
                        (article_span.0, typedecl.span.1),
                    ))
                }
            }
            _ => Err(CompilerError {
                kind: ErrorCode::InvalidTypeDeclaration,
                message: format!("'{}' is geen type", typedecl.value),
                span: typedecl.span,
                span_message: format!("'{}' is geen bekend type ", typedecl.value),
            }),
        }
    }
}

pub(crate) fn is_correct_article(article: &Token, token_type: &Token) -> bool {
    match token_type.type_ {
        TokenType::KWSom => article.type_ == TokenType::KWArticleDefD,
        TokenType::KWVerschil | TokenType::KWProduct | TokenType::KWQuotient => {
            article.type_ == TokenType::KWArticleDefH
        }
        _ => unimplemented!(),
    }
}

impl PolderType {
    pub(crate) fn article_for(type_: &Self) -> TokenType {
        match type_ {
            PolderType::Boolean | PolderType::Char | PolderType::String => TokenType::KWArticleDefD,
            PolderType::Integer | PolderType::Float => TokenType::KWArticleDefH,
        }
    }
}
