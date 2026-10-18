pub(crate) mod token;

use std::str::Chars;

use log::trace;
use peekable_next::{PeekNext, PeekableNext};
use token::{KEYWORDS, Token, TokenType};

use crate::lang::compiler::{CompilerError, ErrorCode, numeric_literal::try_parse_numeric_literal};

macro_rules! make_token {
    ($type_:ident, $value:expr, $span:expr) => {
        Token {
            type_: TokenType::$type_,
            value: $value,
            span: $span,
        }
    };
}

pub(crate) struct Lexer<'a> {
    pub(crate) chars: PeekableNext<Chars<'a>>,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn lex(input: &'a str) -> Result<Vec<Token>, CompilerError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();

        while let Some(token) = lexer.next()? {
            tokens.push(token);
        }

        if log::max_level() >= log::LevelFilter::Trace {
            trace!("{:^16}|{:^16} @ {}", "Type", "Literal", "Position");
            trace!("{:-<1$}", "", 40);
            tokens.iter().for_each(|token| trace!("{}", token));
        }

        Ok(tokens)
    }

    fn new(input: &'a str) -> Self {
        let chars = input.chars().peekable_next();
        Lexer { chars, position: 0 }
    }

    fn next(&mut self) -> Result<Option<Token>, CompilerError> {
        self.skip_whitespace();

        let start_position = self.position;

        match self.read_char() {
            Some('„') | Some('“') | Some('"') => self.build_string(start_position),
            Some('\'') => self.build_char(start_position),
            Some('.') => Ok(Some(make_token!(
                FullStop,
                String::from("."),
                start_position..self.position
            ))),
            Some(c) if c.is_alphabetic() => self.build_identifier(start_position, c),
            Some(c) if c.is_ascii_digit() => self.build_number(start_position, c),
            Some(c) => Err(CompilerError {
                kind: ErrorCode::UnexpectedCharacter,
                message: format!("Compiler doesn't know '{}'", c),
                span_message: format!("{} is not a valid character", c),
                span: self.position - 1..self.position,
            }),
            None => Ok(None),
        }
    }

    fn skip_whitespace(&mut self) -> () {
        while let Some(_) = self.chars.next_if(|c| c.is_whitespace()) {
            self.position += 1;
        }
    }

    fn read_char(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn expect_char(
        &mut self,
        expected: &[char],
        error_message: &str,
    ) -> Result<char, CompilerError> {
        let char = self.read_char();

        if char.is_none_or(|c| !expected.contains(&c)) {
            Err(CompilerError {
                kind: ErrorCode::UnexpectedCharacter,
                message: error_message.to_string(),
                span_message: error_message.to_string(),
                span: self.position - 1..self.position,
            })
        } else {
            Ok(char.unwrap())
        }
    }

    fn continue_while(&mut self, c: char, f: impl Fn(&char) -> bool) -> Vec<char> {
        let mut v = vec![c];
        while let Some(c) = self.chars.next_if(&f) {
            self.position += 1;
            v.push(c);
        }

        v
    }

    fn build_string(&mut self, start_position: usize) -> Result<Option<Token>, CompilerError> {
        let start_char = self.read_char().unwrap();
        // @TODO: Quote escaping is not supported here - either support escaping with backslashes, or support escaping by using the
        // other Unicode character than the one we started the string with
        let str = self.continue_while(start_char, |c| *c != '"' && *c != '”');
        self.expect_char(
            &['"', '”'],
            "Expected closing apostrophe before end of file",
        )?;

        let text = str.into_iter().collect::<String>();
        Ok(Some(make_token!(
            StringLiteral,
            text,
            start_position..self.position
        )))
    }

    fn build_identifier(
        &mut self,
        start_position: usize,
        c: char,
    ) -> Result<Option<Token>, CompilerError> {
        let str = self.continue_while(c, |c| c.is_alphanumeric() || *c == '_');
        let ident = str.into_iter().collect::<String>();
        if let Some(number_word) = try_parse_numeric_literal(&ident, start_position..self.position)
        {
            return Ok(Some(number_word));
        }
        if let Some(keyword) = KEYWORDS.get(ident.as_str()) {
            Ok(Some(Token {
                type_: *keyword,
                value: ident,
                span: start_position..self.position,
            }))
        } else {
            Ok(Some(make_token!(
                Identifier,
                ident,
                start_position..self.position
            )))
        }
    }

    fn build_number(
        &mut self,
        start_position: usize,
        c: char,
    ) -> Result<Option<Token>, CompilerError> {
        let mut str = self.continue_while(c, |c| c.is_ascii_digit());
        if self.chars.peek().is_some_and(|c| *c == ',')
            && self.chars.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            let comma = self.read_char().unwrap();
            str.extend(self.continue_while(comma, |c| c.is_ascii_digit()));
            Ok(Some(make_token!(
                FloatLiteral,
                str.iter().collect::<String>(),
                start_position..self.position
            )))
        } else {
            Ok(Some(make_token!(
                IntegerLiteral,
                str.iter().collect::<String>(),
                start_position..self.position
            )))
        }
    }

    fn build_char(&mut self, start_position: usize) -> Result<Option<Token>, CompilerError> {
        let char = self.read_char();
        if char.is_none() {
            return Err(CompilerError {
                kind: ErrorCode::UnexpectedCharacter,
                message: "Expected a character after single quote".to_string(),
                span_message: "Expected a character here".to_string(),
                span: self.position - 1..self.position,
            });
        }

        self.expect_char(&['\''], "Expected a single quote (') here")?;

        Ok(Some(make_token!(
            CharLiteral,
            char.unwrap().to_string(),
            start_position..self.position
        )))
    }
}
