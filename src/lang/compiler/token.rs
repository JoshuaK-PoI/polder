use std::iter::Peekable;

use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, CompilerResult,
    unshift::{Unshift, UnshiftExpect},
};

pub(crate) type Span = std::ops::Range<usize>;
pub(crate) type TokenStream<'a> = Peekable<std::slice::IterMut<'a, Token>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum TokenType {
    // Special tokens
    Unknown,

    // Single char
    Apostrophe,
    LowerApostrophe,
    FullStop,
    Quote,

    // Keywords
    KWDruk,
    KWAf,
    KWLaat,
    KWZijn,
    KWMaak,
    KWVan,
    KWSchrijf,
    KWOp,
    KWLees,
    KWIn,

    KWArticleIndef,
    KWArticleDefH,
    KWArticleDefD,

    KWConst,

    // Literals
    TrueLiteral,
    FalseLiteral,
    StringLiteral,
    IntegerLiteral,
    FloatLiteral,
    CharLiteral,

    Identifier,
}

pub(crate) const ARTICLES: &[TokenType] = &[
    TokenType::KWArticleIndef,
    TokenType::KWArticleDefH,
    TokenType::KWArticleDefD,
];

pub(crate) const LITERALS: &[TokenType] = &[
    TokenType::StringLiteral,
    TokenType::FloatLiteral,
    TokenType::IntegerLiteral,
    TokenType::TrueLiteral,
    TokenType::FalseLiteral,
    TokenType::CharLiteral,
];

pub(crate) static KEYWORDS: phf::Map<&'static str, TokenType> = phf::phf_map! {
    "Druk" => TokenType::KWDruk,
    "af" => TokenType::KWAf,
    "Schrijf" => TokenType::KWSchrijf,
    "op" => TokenType::KWOp,
    "Lees" => TokenType::KWLees,
    "in" => TokenType::KWIn,
    "Laat" => TokenType::KWLaat,
    "zijn" => TokenType::KWZijn,
    "Maak" => TokenType::KWMaak,
    "van" => TokenType::KWVan,
    "de" => TokenType::KWArticleDefD,
    "het" => TokenType::KWArticleDefH,
    "een" => TokenType::KWArticleIndef,
    "waar" => TokenType::TrueLiteral,
    "onwaar" => TokenType::FalseLiteral,
    "voorgoed" => TokenType::KWConst,
};

impl From<TokenType> for String {
    fn from(token: TokenType) -> String {
        format!("{:?}", token)
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Token {
    pub(crate) type_: TokenType,
    pub(crate) value: String,
    pub(crate) span: Span,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<16.16} {:<16.16} @ {:#04}..{:#04} ",
            format!("{}", self.type_),
            format!("'{}'", self.value),
            self.span.start,
            self.span.end
        )
    }
}

impl<'a> UnshiftExpect<Token, TokenType, CompilerError> for TokenStream<'a> {
    fn unshift_expect(&mut self, expected: TokenType) -> CompilerResult<Token> {
        match self.peek() {
            Some(token) if token.type_ == expected => Ok(self.unshift().unwrap().to_owned()),
            Some(token) => Err(CompilerError {
                kind: crate::lang::compiler::ErrorCode::UnshiftedUnexpectedToken,
                message: format!("Expected '{}', got {}", expected, token),
                span_message: format!("Expected '{}', got {}", expected, token),
                span: self.unshift().unwrap().span.clone(),
            }),
            None => Err(CompilerError {
                kind: crate::lang::compiler::ErrorCode::NoTokensLeft,
                message: format!("Expected token {}, got None", expected),
                span_message: format!("Expected token {}, got None", expected),
                span: 0..0,
            }),
        }
    }

    fn unshift_expect_any(&mut self, expected: &[TokenType]) -> CompilerResult<Token> {
        match self.peek() {
            Some(token) if expected.contains(&token.type_) => {
                Ok(self.unshift().unwrap().to_owned())
            }
            Some(token) => Err(CompilerError {
                kind: crate::lang::compiler::ErrorCode::UnshiftedUnexpectedToken,
                message: format!("Expected one of: {:?}, got {}", expected, token),
                span_message: format!("Expected one of: {:?}, got {}", expected, token),
                span: self.peek().unwrap().span.clone(),
            }),
            None => Err(CompilerError {
                kind: crate::lang::compiler::ErrorCode::NoTokensLeft,
                message: format!("Expected token {:?}, got None", expected),
                span_message: format!("Expected one of: {:?}, got None", expected),
                span: 0..0,
            }),
        }
    }

    fn unshift_if(&mut self, token_type: TokenType) -> Option<&mut Token> {
        match self.peek() {
            Some(token) if token.type_ == token_type => Some(self.unshift().unwrap()),
            _ => None,
        }
    }

    fn next_matches(&mut self, token_type: TokenType) -> bool {
        match self.peek() {
            Some(token) => token.type_ == token_type,
            None => false,
        }
    }

    fn next_matches_any(&mut self, expected: &[TokenType]) -> bool {
        match self.peek() {
            Some(token) => expected.contains(&token.type_),
            None => false,
        }
    }
}
