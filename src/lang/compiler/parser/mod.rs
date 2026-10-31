pub(crate) mod expression;
pub(crate) mod intrinsic;
pub(crate) mod statement;
pub(crate) mod types;

use crate::lang::compiler::{
    CompilerError, ErrorCode,
    lexer::token::{Token, TokenType},
};

use super::CompilerResult;
use statement::Statement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserMode {
    Normal,
    InThenStatement,
}

pub(crate) struct Parser {
    tokens: Vec<Token>,
    position: usize,
    mode: ParserMode,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            mode: ParserMode::Normal,
        }
    }

    fn peek(&self) -> Option<&Token> {
        match self.tokens.get(self.position) {
            Some(token) if token.type_ == TokenType::EOF => None,
            Some(token) => Some(token),
            None => unreachable!(),
        }
    }

    fn next_is(&self, token_type: TokenType) -> bool {
        match self.tokens.get(self.position) {
            Some(token) if token.type_ == token_type => true,
            _ => false,
        }
    }

    fn next_is_any_of(&self, token_types: &[TokenType]) -> bool {
        match self.tokens.get(self.position) {
            Some(token) if token_types.contains(&token.type_) => true,
            _ => false,
        }
    }

    fn next_nth_is_any_of(&self, lookahead: isize, token_types: &[TokenType]) -> bool {
        match self
            .tokens
            .get(std::cmp::max(0, self.position as isize + (lookahead - 1)) as usize)
        {
            Some(token) if token_types.contains(&token.type_) => true,
            _ => false,
        }
    }

    fn next_is_then_unshift(&mut self, token_type: TokenType) -> bool {
        match self.tokens.get(self.position) {
            Some(token) if token.type_ == token_type => {
                self.position += 1;
                true
            }
            _ => false,
        }
    }

    fn expect(&mut self, token_type: TokenType) -> CompilerResult<Token> {
        match self.tokens.get(self.position) {
            Some(token) if token.type_ == token_type => Ok(self.next()),
            Some(token) => Err(CompilerError::new(
                ErrorCode::UnshiftedUnexpectedToken,
                format!("Onverwacht: '{}'", token.value),
                token.span,
            )
            .with_span_message(format!("Hier verwachtte ik '{}'.", token_type))),
            None => unreachable!(),
        }
    }

    fn expect_any_of(&mut self, token_types: &[TokenType]) -> CompilerResult<Token> {
        match self.tokens.get(self.position) {
            Some(token) if token_types.contains(&token.type_) => Ok(self.next()),
            Some(token) => Err(CompilerError::new(
                crate::lang::compiler::ErrorCode::UnshiftedUnexpectedToken,
                format!("Expected one of: {:?}, got {}", token_types, token.value),
                self.peek().unwrap().span,
            )),
            None => unreachable!(),
        }
    }

    #[must_use]
    fn next(&mut self) -> Token {
        match self.tokens.get(self.position) {
            Some(token) if token.type_ != TokenType::EOF => {
                self.position += 1;
                token.clone()
            }
            _ => panic!(
                "Unshifted token where there was none, at position {} (out of bounds)",
                self.position
            ),
        }
    }

    fn get_nth(&self, lookahead: isize) -> &Token {
        match self
            .tokens
            .get(std::cmp::max(0, self.position as isize + (lookahead - 1)) as usize)
        {
            Some(token) if token.type_ != TokenType::EOF => token,
            _ => panic!(
                "Unshifted token where there was none, at position {} (out of bounds)",
                self.position as isize + (lookahead - 1)
            ),
        }
    }

    pub(crate) fn parse(tokens: Vec<Token>) -> CompilerResult<Vec<Statement>> {
        let mut parser = Parser::new(tokens);
        let mut ast = Vec::new();

        while parser.peek().is_some() {
            ast.push(parser.stmt()?)
        }

        ast.retain(|stmt| *stmt != Statement::Comment);

        Ok(ast)
    }
}
