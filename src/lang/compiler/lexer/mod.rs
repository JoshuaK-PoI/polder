pub(crate) mod token;

use log::trace;
use token::{KEYWORDS, Token, TokenType};

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode, numeric_literal::try_parse_numeric_literal,
};

macro_rules! make_token {
    ($type_:ident, $value:expr, $span:expr) => {
        Token {
            type_: TokenType::$type_,
            value: $value,
            span: $span,
        }
    };
}

pub(crate) struct Lexer {
    pub(crate) chars: Vec<char>,
    position: usize,
}

impl Lexer {
    pub(crate) fn lex(input: &str) -> Result<Vec<Token>, CompilerError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();

        while let Some(token) = lexer.next_token()? {
            tokens.push(token);
        }

        tokens.push(Token {
            type_: TokenType::EOF,
            value: format!("<<Einde van het bestand>>"),
            span: (lexer.position, lexer.position),
        });

        if log::max_level() >= log::LevelFilter::Trace {
            trace!("{:^16}|{:^32} @ {}", "Type", "Literal", "Position");
            trace!("{:-<1$}", "", 40);
            tokens.iter().for_each(|token| trace!("{}", token));
        }

        Ok(tokens)
    }

    fn new(input: &str) -> Self {
        let chars = input.chars().collect();
        Lexer { chars, position: 0 }
    }

    fn next_token(&mut self) -> Result<Option<Token>, CompilerError> {
        self.skip_whitespace();

        let start_position = self.position;

        match self.peek() {
            Some('„') | Some('“') | Some('"') => self.build_string(start_position),
            Some('\'') => self.build_char(start_position),
            Some('.') => {
                self.next();
                Ok(Some(make_token!(
                    FullStop,
                    String::from("."),
                    (start_position, self.position)
                )))
            }
            Some(',') => {
                self.next();
                Ok(Some(make_token!(
                    Comma,
                    String::from(","),
                    (start_position, self.position)
                )))
            }
            Some(':') => {
                self.next();
                Ok(Some(make_token!(
                    Colon,
                    String::from(":"),
                    (start_position, self.position)
                )))
            }
            Some(c) if c.is_alphabetic() => self.build_identifier(start_position),
            Some(c) if c.is_ascii_digit() => self.build_number(start_position),
            Some(c) => Err(CompilerError {
                kind: ErrorCode::UnexpectedCharacter,
                message: format!("Compiler doesn't know '{}'", c),
                span_message: format!("{} is not a valid character", c),
                span: (self.position - 1, self.position),
            }),
            None => Ok(None),
        }
    }

    fn skip_whitespace(&mut self) -> () {
        if self.peek().is_some_and(|c| c.is_whitespace()) {
            self.continue_while(|c| c.is_whitespace());
        }
    }

    fn expect_char(
        &mut self,
        expected: &[char],
        error_message: &str,
    ) -> Result<char, CompilerError> {
        let char = self.next();

        if char.is_none_or(|c| !expected.contains(&c)) {
            Err(CompilerError {
                kind: ErrorCode::UnexpectedCharacter,
                message: error_message.to_string(),
                span_message: error_message.to_string(),
                span: (self.position - 1, self.position),
            })
        } else {
            Ok(char.unwrap().to_owned())
        }
    }

    fn continue_while(&mut self, f: impl Fn(&char) -> bool) -> Vec<char> {
        let mut v = vec![*self.next().unwrap()];
        while let Some(c) = self.next_if(&f) {
            v.push(*c);
        }
        v
    }

    fn build_string(&mut self, start_position: usize) -> Result<Option<Token>, CompilerError> {
        self.next(); // Consume opening quote
        // @TODO: Quote escaping is not supported here - either support escaping with backslashes, or support escaping by using the
        // other Unicode character than the one we started the string with
        let str = self.continue_while(|c| *c != '"' && *c != '”');
        self.expect_char(
            &['"', '”'],
            "Expected closing apostrophe before end of file",
        )?;

        let text = str.into_iter().collect::<String>();
        Ok(Some(make_token!(
            StringLiteral,
            text,
            (start_position, self.position)
        )))
    }

    fn build_identifier(&mut self, start_position: usize) -> Result<Option<Token>, CompilerError> {
        if self.peek() == Some(&'P')
            && self.peek_nth(1) == Some(&'.')
            && self.peek_nth(2) == Some(&'S')
            && self.peek_nth(3) == Some(&'.')
        {
            let comment = self.parse_comment()?;
            return Ok(Some(make_token!(
                Comment,
                comment,
                (start_position, self.position)
            )));
        }
        let str = self.continue_while(|c| c.is_alphanumeric() || *c == '_');
        let ident = str.into_iter().collect::<String>();
        if let Some(number_word) =
            try_parse_numeric_literal(&ident.to_ascii_lowercase(), (start_position, self.position))
        {
            return Ok(Some(number_word));
        }

        if let Some(keyword) = KEYWORDS.get(&ident.to_ascii_lowercase()) {
            Ok(Some(Token {
                type_: *keyword,
                value: ident,
                span: (start_position, self.position),
            }))
        } else {
            Ok(Some(make_token!(
                Identifier,
                ident,
                (start_position, self.position)
            )))
        }
    }

    fn build_number(&mut self, start_position: usize) -> Result<Option<Token>, CompilerError> {
        let mut str = self.continue_while(|c| c.is_ascii_digit());
        if self.peek().is_some_and(|c| *c == ',')
            && self.peek_nth(1).is_some_and(|c| c.is_ascii_digit())
        {
            str.extend(self.continue_while(|c| c.is_ascii_digit()));
            Ok(Some(make_token!(
                FloatLiteral,
                str.iter().collect::<String>(),
                (start_position, self.position)
            )))
        } else {
            Ok(Some(make_token!(
                IntegerLiteral,
                str.iter().collect::<String>(),
                (start_position, self.position)
            )))
        }
    }

    fn build_char(&mut self, start_position: usize) -> Result<Option<Token>, CompilerError> {
        self.next(); // Consume quote
        let char = match self.next() {
            None => Err(CompilerError {
                kind: ErrorCode::UnexpectedCharacter,
                message: "Expected a character after single quote".to_string(),
                span_message: "Expected a character here".to_string(),
                span: (self.position - 1, self.position),
            }),
            Some(ch) => Ok(ch.to_owned()),
        }?;
        self.expect_char(&['\''], "Expected a single quote (') here")?;

        Ok(Some(make_token!(
            CharLiteral,
            char.to_string(),
            (start_position, self.position)
        )))
    }

    fn parse_comment(&mut self) -> CompilerResult<String> {
        let mut str = String::new();
        str.push(*self.next().expect("Expected 'P'"));
        str.push(*self.next().expect("Expected '.'"));
        str.push(*self.next().expect("Expected 'S'"));
        str.push(*self.next().expect("Expected '.'"));
        while let Some(c) = self.peek() {
            if c == &'\n' {
                break;
            }
            str.push(*self.next().unwrap());
        }

        Ok(str)
    }

    fn next_if(&mut self, predicate: impl Fn(&char) -> bool) -> Option<&char> {
        match self.chars.get(self.position) {
            Some(ch) if predicate(ch) => {
                self.position += 1;
                Some(ch)
            }
            _ => None,
        }
    }

    fn next(&mut self) -> Option<&char> {
        let ch = self.chars.get(self.position);
        self.position += 1;
        ch
    }

    fn peek(&self) -> Option<&char> {
        self.chars.get(self.position)
    }

    fn peek_nth(&self, offset: isize) -> Option<&char> {
        self.chars
            .get(std::cmp::max(0, self.position as isize + offset) as usize)
    }
}
