use std::{iter::Peekable, str::Chars};

use crate::lang::compiler::token::{KEYWORDS, Token, TokenStream, TokenType};

pub(crate) struct Lexer<'a> {
    pub(crate) chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    position: usize,
}

macro_rules! make_token {
    ($type_:ident, $value:expr, $span:expr) => {
        Token {
            type_: TokenType::$type_,
            value: $value,
            span: $span,
        }
    };
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let start_position = self.position;

        match self.read_char() {
            Some('„') => self.build_string(start_position, '„'),
            Some('”') => self.build_string(start_position, '”'),
            Some('"') => self.build_string(start_position, '"'),
            Some('.') => Some(make_token!(
                FullStop,
                String::from("."),
                start_position..self.position
            )),
            Some(c) if c.is_alphabetic() => self.build_identifier(start_position, c),
            Some(c) if c.is_ascii_digit() => self.build_integer_literal(start_position, c),
            Some(c) => Some(make_token!(
                Unknown,
                String::from(c),
                start_position..self.position
            )),
            None => return None,
        }
    }
}

impl<'a> Lexer<'a> {
    pub(crate) fn new() -> Lexer<'a> {
        Lexer {
            chars: "".chars().peekable(),
            tokens: Vec::new(),
            position: 0,
        }
    }

    pub(crate) fn lex(&mut self, input: &'a str, tokens: &mut Vec<Token>) -> () {
        self.chars = input.chars().peekable();
        while let Some(token) = self.next() {
            tokens.push(token);
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

    fn continue_while(&mut self, c: char, f: impl Fn(&char) -> bool) -> Vec<char> {
        let mut v = vec![c];
        while let Some(c) = self.chars.next_if(&f) {
            self.position += 1;
            v.push(c);
        }

        v
    }

    pub(crate) fn get_tokens_peekable(&'a mut self) -> TokenStream<'a> {
        self.tokens.iter_mut().peekable()
    }

    fn build_string(&mut self, start_position: usize, start_char: char) -> Option<Token> {
        let mut str = self.continue_while(start_char, |c| *c != '"' && *c != '”');
        let closing_apos = self.read_char().unwrap();
        str.push(closing_apos);
        let text = str.into_iter().collect::<String>();
        Some(make_token!(
            StringLiteral,
            text,
            start_position..self.position
        ))
    }

    fn build_identifier(&mut self, start_position: usize, c: char) -> Option<Token> {
        let str = self.continue_while(c, |c| c.is_alphanumeric() || *c == '_');
        let ident = str.into_iter().collect::<String>();
        if let Some(keyword) = KEYWORDS.get(ident.as_str()) {
            Some(Token {
                type_: *keyword,
                value: ident,
                span: start_position..self.position,
            })
        } else {
            Some(make_token!(
                Identifier,
                ident,
                start_position..self.position
            ))
        }
    }

    fn build_integer_literal(&mut self, start_position: usize, c: char) -> Option<Token> {
        let str = self.continue_while(c, |c| c.is_ascii_digit());
        let value = str.into_iter().collect::<String>();
        Some(make_token!(
            IntegerLiteral,
            value,
            start_position..self.position
        ))
    }
}
