use serde::Serialize;

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
    KWVoeg,
    KWToe,

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
    KWVoor,
    KWElke,

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
    EOF,
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
    "voeg" => TokenType::KWVoeg,
    "toe" => TokenType::KWToe,
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
    "voor" => TokenType::KWVoor,
    "elke" => TokenType::KWElke,
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
                TokenType::EOF => "<EOF>",
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
                TokenType::KWVoeg => "voeg",
                TokenType::KWToe => "toe",
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
                TokenType::KWVoor => "voor",
                TokenType::KWElke => "elke",
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
