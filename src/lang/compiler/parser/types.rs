use serde::Serialize;

use super::super::{CompilerError, ErrorCode, lexer::token::TokenType};
use crate::lang::compiler::{
    CompilerResult,
    lexer::token::{ARTICLES, Token},
    parser::Parser,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) enum PolderType {
    Boolean,
    String,
    Integer,
    Float,
    Char,
    Array(Option<Box<PolderType>>),
}

impl From<&String> for PolderType {
    fn from(value: &String) -> Self {
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
    pub(crate) fn is_valid_type(value: &String) -> bool {
        match value.as_str() {
            "getal" => true,
            "kommagetal" => true,
            "waarheid" => true,
            "tekst" => true,
            "letter" => true,
            _ => false,
        }
    }
    pub(crate) fn from_pluralized(pluralized_value: &String) -> Option<PolderType> {
        match pluralized_value.as_str() {
            "getallen" => Some(PolderType::Integer),
            "kommagetallen" => Some(PolderType::Float),
            "waarheden" => Some(PolderType::Boolean),
            "teksten" => Some(PolderType::String),
            "letters" => Some(PolderType::Char),
            _ => None,
        }
    }

    pub(crate) fn to_pluralized(&self) -> String {
        match self {
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

    /**
     * Get the associated article ('de' / 'het') for the current type.
     */
    pub(crate) fn article(&self) -> TokenType {
        match self {
            PolderType::Boolean | PolderType::Char | PolderType::String | PolderType::Array(_) => {
                TokenType::KWArticleDefD
            }
            PolderType::Integer | PolderType::Float => TokenType::KWArticleDefH,
        }
    }

    /**
     * Gets the inner value from a PolderType::Array().
     * If the given type is not an Array, or its inner type is not set, this returns None.
     */
    pub(crate) fn get_array_type(&self) -> Option<&PolderType> {
        match self {
            PolderType::Array(arr) => arr.as_deref(),
            _ => None
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
    pub(super) fn try_get_type(&mut self) -> CompilerResult<Option<PolderType>> {
        if self.next_is(TokenType::KWArticleIndef) {
            // We must have a type; an indefinite article cannot be followed by an expression, only a valid type name
            let article = self.expect(TokenType::KWArticleIndef)?;
            let type_ = match self.parse_type(&article)? {
                Some(type_) => Ok(Some(type_)),
                _ => Err(CompilerError::new(
                    ErrorCode::InvalidTypeDeclaration,
                    format!("Ongeldig type: {}", self.get_nth(1).value),
                    self.get_nth(1).span,
                )
                .with_span_message(format!("Verwachtte hier een geldig type, maar vond '{}'", self.get_nth(1).value))
                .with_help(format!("Een variabele die aangemaakt wordt met 'een', moet een geldig type hebben. \
                    Als je een variabele wilt aanmaken met een initiele waarde, kan je het lidwoord weglaten.
                "))),
            }?;

            if !self.next_is(TokenType::KWZijn) {
                return Err(CompilerError::new(
                    ErrorCode::IndefiniteCannotHaveValue,
                    format!("'een {}' kan geen waarde hebben", &type_.as_ref().unwrap()),
                    self.peek().unwrap().span,
                )
                .with_span_message(format!(
                    "Verwachtte hier 'zijn', maar vond {}",
                    self.peek().unwrap().value
                ))
                .with_help(format!(
                    "Gebruik '{} {}' als je een waarde opgeeft.",
                    &type_.as_ref().unwrap().article(),
                    type_.unwrap()
                )));
            } else {
                Ok(type_)
            }
        } else if self.next_is_any_of(&[TokenType::KWArticleDefD, TokenType::KWArticleDefH])
            && (PolderType::is_valid_type(&self.get_nth(2).value)
                || self.get_nth(2).value == "lijst".to_string())
        {
            // Next token could be an article and a type after that, but some expressions also start with an article
            // so we can't be sure.
            // If we match an article then a non-type keyword; we don't throw an error and let the initializer expression handler
            // continue the parsing.
            let article = self.expect_any_of(ARTICLES)?;
            let type_ = self.parse_type(&article)?;
            if self.next_is(TokenType::KWZijn) {
                Err(CompilerError::new(
                    ErrorCode::DefiniteMustHaveValue,
                    format!("Verwachtte een expressie, maar vond 'zijn'."),
                    self.peek().unwrap().span,
                ).with_help(format!("Als je een variabele wilt aanmaken zonder initiele waarde, gebruik dan 'een {}'", type_.unwrap())))
            } else {
                Ok(type_)
            }
        } else {
            Ok(None)
        }
    }

    fn parse_type(&mut self, article: &Token) -> CompilerResult<Option<PolderType>> {
        let typedecl = self.peek();
        if typedecl.is_none() {
            return Err(CompilerError::new(
                ErrorCode::NoTokensLeft,
                format!("Verwachtte een type, maar het einde van het bestand was bereikt"),
                self.get_nth(0).span,
            ));
        }
        let typedecl = typedecl.unwrap();

        match typedecl.value.as_str() {
            "getal" | "kommagetal" | "waarheid" | "tekst" | "letter" => {
                let type_ = PolderType::from(&typedecl.value);
                if !(article.type_ == TokenType::KWArticleIndef || article.type_ == type_.article())
                {
                    Err(CompilerError::new(
                        ErrorCode::WrongArticleForType,
                        format!("Lekker genederlandst"),
                        article.span,
                    )
                    .with_help(format!(
                        "Bedoelde je wellicht '{} {}'?",
                        type_.article(),
                        type_
                    )))
                } else {
                    let _ = self.next();
                    Ok(Some(type_))
                }
            }
            "lijst" => self.try_parse_array_type(article),
            _ => Ok(None),
        }
    }

    fn try_parse_array_type(&mut self, article: &Token) -> CompilerResult<Option<PolderType>> {
        self.expect(TokenType::Identifier)?;
        self.expect(TokenType::KWVan)?;
        if article.type_ == TokenType::KWArticleIndef {
            let inner_type_ident = self
            .get_identifier()
            .map_err(|err| {
                CompilerError::new(
                    err.kind,
                    format!("Verwachtte een type voor de lijst na 'een'"),
                    err.span
                ).with_help(
                    "Een lijst die aangemaakt wordt met een onbepaald lidwoord moet een type hebben, \
                    en kan niet direct een waarde toegewezen krijgen. Om een waarde toe te wijzen, \
                    gebruik 'de lijst' in plaats van 'een lijst'.".to_string()
                )
            })?
            .value;
            let inner_type = match PolderType::from_pluralized(&inner_type_ident) {
            Some(type_) => Ok(Some(PolderType::Array(Some(Box::new(type_))))),
            None => Err(CompilerError::new(
                ErrorCode::MismatchedType,
                format!("Verwachtte een type voor de lijst na 'een', maar vond {}", inner_type_ident),
                self.get_nth(0).span,
            ).with_help(
                "Een lijst die aangemaakt wordt met een onbepaald lidwoord moet een type hebben, \
                en kan niet direct een waarde toegewezen krijgen. Om een waarde toe te wijzen, \
                gebruik 'de lijst' in plaats van 'een lijst'.".to_string()
            ))
        }?;

            Ok(inner_type)
        } else if [TokenType::KWArticleDefD, TokenType::KWArticleDefH].contains(&article.type_) {
            // If the article is definite, inner_type may be set as a valid plural type; in any case an initializer list is required.
            let possible_inner_type = self.get_identifier();
            if possible_inner_type.is_err() {
                return Ok(Some(PolderType::Array(None)));
            } else if PolderType::from_pluralized(&possible_inner_type.as_ref().unwrap().value)
                .is_some()
            {
                Ok(Some(PolderType::Array(Some(Box::new(
                    PolderType::from_pluralized(&possible_inner_type.unwrap().value).unwrap(),
                )))))
            } else {
                panic!(
                    "Found a definite article when parsing an array type, but could not match to a valid token after it"
                )
            }
        } else {
            panic!(
                "Tried to parse an array declaration, but got something other than an article, which we did not expect"
            )
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
