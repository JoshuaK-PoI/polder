pub(crate) mod token;

use std::{iter::Peekable, str::Chars};

use token::{KEYWORDS, Token, TokenType};


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
    pub(crate) chars: Peekable<Chars<'a>>,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Lexer<'a> {
        let chars = input.chars().peekable();
        Lexer {
            chars,
            position: 0,
        }
    }

    pub(crate) fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next() {
            tokens.push(token);
        }

        tokens
    }

    fn next(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let start_position = self.position;

        match self.read_char() {
            Some('„') => self.build_string(start_position, '„'),
            Some('”') => self.build_string(start_position, '”'),
            Some('"') => self.build_string(start_position, '"'),
            Some('\'') => self.build_char(start_position),
            Some('.') => Some(make_token!(
                FullStop,
                String::from("."),
                start_position..self.position
            )),
            Some(c) if c.is_alphabetic() => self.build_identifier(start_position, c),
            Some(c) if c.is_ascii_digit() => self.build_number(start_position, c),
            Some(c) => Some(make_token!(
                Unknown,
                String::from(c),
                start_position..self.position
            )),
            None => return None,
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

    fn build_string(&mut self, start_position: usize, _start_char: char) -> Option<Token> {
        let start_char = self.read_char().unwrap();
        let str = self.continue_while(start_char, |c| *c != '"' && *c != '”');
        let closing_apos = self.read_char();
        if closing_apos.is_none() {
            panic!("Expected closing apostrophe before end of file");
        }

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

    fn build_number(&mut self, start_position: usize, c: char) -> Option<Token> {
        let str = self.continue_while(c, |c| c.is_ascii_digit() || c == &',');
        let value = str.into_iter().collect::<String>();
        let commas = value.chars().filter(|c| c == &',').count();
        if commas > 1 {
            // @TODO Support error handling in lexer
            panic!("Found more than one comma in number allocation")
        }
        if commas == 1 {
            Some(make_token!(
                FloatLiteral,
                value,
                start_position..self.position
            ))
        } else {
            Some(make_token!(
                IntegerLiteral,
                value,
                start_position..self.position
            ))
        }
    }

    fn build_char(&mut self, start_position: usize) -> Option<Token> {
        let char = self.read_char();
        if char.is_none() {
            panic!("Expected a character after single quote");
        }

        let end_quote = self.read_char();
        if end_quote.is_none() {
            panic!("Expected an end quote after character but got nothing")
        } else if end_quote.unwrap() != '\'' {
            panic!(
                "Expected an end quote after a single character, but got '{}'",
                end_quote.unwrap()
            )
        }

        Some(make_token!(
            CharLiteral,
            char.unwrap().to_string(),
            start_position..self.position
        ))
    }
}
