use peekable_next::PeekableNext;
use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, CompilerResult,
    iter::unshift::{Unshift, UnshiftExpect},
};

pub(crate) type TokenStream<'a> = PeekableNext<std::slice::IterMut<'a, Token>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum TokenType {
    // Comment
    Comment,

    // Single char
    Apostrophe,
    LowerApostrophe,
    FullStop,
    Quote,
    Comma,
    Colon,

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

    // Arithmetic KWs
    KWTel,
    KWBij,
    KWSom,
    KWTrek,
    KWVerschil,
    KWVermenigvuldig,
    KWMet,
    KWProduct,
    KWDeel,
    KWDoor,
    KWQuotient,
    KWVerhoog,
    KWVerlaag,

    KWArticleDefD,
    KWArticleDefH,
    KWArticleIndef,

    KWEn,
    KWConst,

    // Conditionals
    KWKleiner,
    KWGroter,
    KWGelijk,
    KWIs,
    KWDan,
    KWAan,

    // Control Flow
    KWAls,
    KWZo,
    KWNiet,
    KWAnders,
    KWZolang,

    // Literals
    TrueLiteral,
    FalseLiteral,
    StringLiteral,
    IntegerLiteral,
    FloatLiteral,
    CharLiteral,

    Identifier,
    KWKlaar,
    KWKees,
}

pub(crate) const BINARY_OPERATORS: &[TokenType] = &[
    TokenType::KWKleiner,
    TokenType::KWGroter,
    TokenType::KWGelijk,
];

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

pub(crate) const ARITH_EXPR: &[TokenType] = &[
    TokenType::KWSom,
    TokenType::KWVerschil,
    TokenType::KWProduct,
    TokenType::KWQuotient,
];

pub(crate) static KEYWORDS: phf::Map<&'static str, TokenType> = phf::phf_map! {
    "druk" => TokenType::KWDruk,
    "af" => TokenType::KWAf,
    "schrijf" => TokenType::KWSchrijf,
    "op" => TokenType::KWOp,
    "lees" => TokenType::KWLees,
    "in" => TokenType::KWIn,
    "laat" => TokenType::KWLaat,
    "zijn" => TokenType::KWZijn,
    "maak" => TokenType::KWMaak,
    "van" => TokenType::KWVan,
    "tel" => TokenType::KWTel,
    "bij" => TokenType::KWBij,
    "som" => TokenType::KWSom,
    "trek" => TokenType::KWTrek,
    "verschil" => TokenType::KWVerschil,
    "vermenigvuldig" => TokenType::KWVermenigvuldig,
    "met" => TokenType::KWMet,
    "product" => TokenType::KWProduct,
    "deel" => TokenType::KWDeel,
    "door" => TokenType::KWDoor,
    "quotiënt" => TokenType::KWQuotient,
    "verhoog" => TokenType::KWVerhoog,
    "verlaag" => TokenType::KWVerlaag,
    "kleiner" => TokenType::KWKleiner,
    "groter" => TokenType::KWGroter,
    "gelijk" => TokenType::KWGelijk,
    "is" => TokenType::KWIs,
    "dan" => TokenType::KWDan,
    "aan" => TokenType::KWAan,
    "als" => TokenType::KWAls,
    "zo" => TokenType::KWZo,
    "niet" => TokenType::KWNiet,
    "anders" => TokenType::KWAnders,
    "zolang" => TokenType::KWZolang,
    "klaar" => TokenType::KWKlaar,
    "kees" => TokenType::KWKees,
    "de" => TokenType::KWArticleDefD,
    "het" => TokenType::KWArticleDefH,
    "een" => TokenType::KWArticleIndef,
    "en" => TokenType::KWEn,
    "voorgoed" => TokenType::KWConst,
    "waar" => TokenType::TrueLiteral,
    "onwaar" => TokenType::FalseLiteral,
};

impl From<TokenType> for String {
    fn from(token: TokenType) -> String {
        format!(
            "{}",
            match token {
                TokenType::Comment => "<Comment>",
                TokenType::Apostrophe => "”",
                TokenType::LowerApostrophe => "„",
                TokenType::FullStop => ".",
                TokenType::Quote => "'",
                TokenType::Comma => ",",
                TokenType::Colon => ":",
                TokenType::KWDruk => "druk",
                TokenType::KWAf => "af",
                TokenType::KWLaat => "laat",
                TokenType::KWZijn => "zijn",
                TokenType::KWMaak => "maak",
                TokenType::KWVan => "van",
                TokenType::KWSchrijf => "schrijf",
                TokenType::KWOp => "op",
                TokenType::KWLees => "lees",
                TokenType::KWIn => "in",
                TokenType::KWTel => "tel",
                TokenType::KWBij => "bij",
                TokenType::KWSom => "som",
                TokenType::KWTrek => "trek",
                TokenType::KWVerschil => "verschil",
                TokenType::KWVermenigvuldig => "vermenigvuldig",
                TokenType::KWMet => "met",
                TokenType::KWProduct => "product",
                TokenType::KWDeel => "deel",
                TokenType::KWDoor => "door",
                TokenType::KWQuotient => "quotiënt",
                TokenType::KWVerhoog => "verhoog",
                TokenType::KWVerlaag => "verlaag",
                TokenType::KWKleiner => "kleiner",
                TokenType::KWGroter => "groter",
                TokenType::KWGelijk => "gelijk",
                TokenType::KWIs => "is",
                TokenType::KWDan => "dan",
                TokenType::KWAan => "aan",
                TokenType::KWAls => "als",
                TokenType::KWZo => "zo",
                TokenType::KWNiet => "niet",
                TokenType::KWAnders => "anders",
                TokenType::KWZolang => "zolang",
                TokenType::KWKlaar => "klaar",
                TokenType::KWKees => "Kees",
                TokenType::KWArticleDefD => "de",
                TokenType::KWArticleDefH => "het",
                TokenType::KWArticleIndef => "een",
                TokenType::KWEn => "en",
                TokenType::KWConst => "voorgoed",
                TokenType::TrueLiteral => "waar",
                TokenType::FalseLiteral => "onwaar",
                TokenType::StringLiteral => "tekst",
                TokenType::IntegerLiteral => "getal",
                TokenType::FloatLiteral => "kommagetal",
                TokenType::CharLiteral => "karakter",
                TokenType::Identifier => "identificatie",
            }
        )
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
    pub(crate) span: (usize, usize),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<16.16} {:<16.16} @ {:#04}..{:#04} ",
            format!("{}", self.type_),
            format!("'{}'", self.value),
            self.span.0,
            self.span.1
        )
    }
}

impl<'a> UnshiftExpect<Token, TokenType, CompilerError> for TokenStream<'a> {
    fn unshift_expect(&mut self, expected: TokenType) -> CompilerResult<Token> {
        match self.peek() {
            Some(token) if token.type_ == expected => Ok(self.unshift().unwrap().to_owned()),
            Some(token) => Err(CompilerError {
                kind: crate::lang::compiler::ErrorCode::UnshiftedUnexpectedToken,
                message: format!("Onverwacht: '{}'", token.value),
                span_message: format!("Hier verwachtte ik '{}'.", expected),
                span: self.unshift().unwrap().span,
            }),
            None => Err(CompilerError {
                kind: crate::lang::compiler::ErrorCode::NoTokensLeft,
                message: format!("Expected token {}, got None", expected),
                span_message: format!("Expected token {}, got None", expected),
                span: (0, 0),
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
                span: self.peek().unwrap().span,
            }),
            None => Err(CompilerError {
                kind: crate::lang::compiler::ErrorCode::NoTokensLeft,
                message: format!("Expected token {:?}, got None", expected),
                span_message: format!("Expected one of: {:?}, got None", expected),
                span: (0, 0),
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

    /// Returns true if the next 2 tokens in the stream match the expected token types provided.
    /// The immediate next token must match a token type of the first element of the `expected` tuple,
    /// the token after that must match a token type of the second element of the tuple.
    fn next_and_after_matches(&mut self, expected: (&[TokenType], &[TokenType])) -> bool {
        match self.peek() {
            Some(token) if expected.0.contains(&token.type_) => match self.peek_next() {
                Some(token_next) if expected.1.contains(&token_next.type_) => true,
                Some(_) => false,
                None => false,
            },
            Some(_) => false,
            None => false,
        }
    }
}
