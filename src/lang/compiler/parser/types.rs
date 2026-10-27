use serde::Serialize;

use super::super::{CompilerError, ErrorCode, lexer::token::TokenType};
use crate::lang::compiler::{CompilerResult, lexer::token::Token, parser::Parser};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) enum PolderType {
    Boolean,
    String,
    Integer,
    Float,
    Char,
    Array(Option<Box<PolderType>>),
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

impl PolderType {
    pub(crate) fn from_pluralized(pluralized_value: String) -> Option<PolderType> {
        match pluralized_value.as_str() {
            "getallen" => Some(PolderType::Integer),
            "kommagetallen" => Some(PolderType::Float),
            "waarheden" => Some(PolderType::Boolean),
            "teksten" => Some(PolderType::String),
            "letters" => Some(PolderType::Char),
            _ => None,
        }
    }

    pub(crate) fn to_pluralized(type_: &PolderType) -> String {
        match type_ {
            PolderType::Integer => format!("getallen"),
            PolderType::Float => format!("kommagetallen"),
            PolderType::Boolean => format!("waarheden"),
            PolderType::String => format!("teksten"),
            PolderType::Char => format!("letters"),
            PolderType::Array(a) => {
                format!(
                    "lijsten van {}",
                    PolderType::to_pluralized(&a.as_ref().unwrap())
                )
            }
        }
    }

    pub(crate) fn get_array_type(
        arr: &Vec<super::expression::literal::Literal>,
    ) -> Box<PolderType> {
        match arr.get(0) {
            Some(lit) => Box::new(lit.into()),
            None => panic!("Cannot get type from array literals; no elements in array"),
        }
    }
}

impl From<&PolderType> for String {
    fn from(value: &PolderType) -> Self {
        format!(
            "{}",
            match value {
                PolderType::Boolean => format!("waarheid"),
                PolderType::String => format!("tekst"),
                PolderType::Integer => format!("getal"),
                PolderType::Float => format!("kommagetal"),
                PolderType::Char => format!("karakter"),
                PolderType::Array(boxed_type) => format!(
                    "lijst van {}",
                    if boxed_type.is_some() {
                        PolderType::to_pluralized(boxed_type.as_ref().unwrap())
                    } else {
                        String::from("<onbekend>")
                    }
                ),
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
    pub(crate) fn parse_type(&mut self) -> CompilerResult<(PolderType, (usize, usize))> {
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
            "lijst" => {
                if ![TokenType::KWArticleDefD, TokenType::KWArticleIndef].contains(&article_type) {
                    Err(CompilerError {
                        kind: ErrorCode::WrongArticleForType,
                        message: format!("Lekker genederlandst"),
                        span_message: format!("Dat is geen correct Nederlands"),
                        span: article_span,
                    })
                } else {
                    self.parse_array_type((article_span.0, typedecl.span.1))
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

    fn parse_array_type(
        &mut self,
        span: (usize, usize),
    ) -> CompilerResult<(PolderType, (usize, usize))> {
        self.expect(TokenType::KWVan)?;

        if self.next_is(TokenType::Identifier) {
            let inner_type = self.next();
            if let Some(inner_type) = PolderType::from_pluralized(inner_type.value.clone()) {
                Ok((PolderType::Array(Some(Box::new(inner_type))), span))
            } else {
                Err(CompilerError {
                    kind: ErrorCode::InvalidTypeDeclaration,
                    message: format!("'{}' is geen type", inner_type.value),
                    span: inner_type.span,
                    span_message: format!("'{}' is geen bekend type ", inner_type.value),
                })
            }
        } else {
            Ok((PolderType::Array(None), span))
        }
    }
}

pub(crate) fn is_correct_article(article: &Token, token_type: &Token) -> bool {
    match token_type.type_ {
        TokenType::KWSom | TokenType::KWModulo => article.type_ == TokenType::KWArticleDefD,
        TokenType::KWVerschil | TokenType::KWProduct | TokenType::KWQuotient => {
            article.type_ == TokenType::KWArticleDefH
        }
        _ => unimplemented!(),
    }
}

impl PolderType {
    pub(crate) fn article_for(type_: &Self) -> TokenType {
        match type_ {
            PolderType::Boolean | PolderType::Char | PolderType::String | PolderType::Array(_) => {
                TokenType::KWArticleDefD
            }
            PolderType::Integer | PolderType::Float => TokenType::KWArticleDefH,
        }
    }
}
