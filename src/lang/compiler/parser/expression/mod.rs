pub(crate) mod binary;
pub(crate) mod literal;

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    iter::unshift::{Unshift, UnshiftExpect},
    lexer::token::{ARITH_EXPR, ARTICLES, BINARY_OPERATORS, LITERALS, TokenStream, TokenType},
    parser::{
        expression::ExpressionType::{Binary, Unary},
        statement::{BinaryOp, UnaryOp},
        types::{PolderType, parse_type},
    },
};

use literal::{Literal, literal};
use serde::Serialize;

const MAX_RECURSION_DEPTH: usize = 256;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Expression {
    pub(crate) expression: ExpressionType,
    pub(crate) span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum ExpressionType {
    Literal(Literal),
    Identifier(String),
    Type(PolderType),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct BinaryExpression {
    pub(crate) op: BinaryOp,
    pub(crate) lhs: Box<Expression>,
    pub(crate) rhs: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct UnaryExpression {
    pub(crate) op: UnaryOp,
    pub(crate) expr: Box<Expression>,
}

impl std::fmt::Display for ExpressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ExpressionType::Binary(_) => "Binary".to_string(),
                ExpressionType::Unary(_) => "Unary".to_string(),
                ExpressionType::Literal(_) => "Literal".to_string(),
                ExpressionType::Identifier(_) => "Identifier".to_string(),
                ExpressionType::Type(_) => "Type".to_string(),
            }
        )
    }
}

impl Expression {
    pub(crate) fn make(expression: ExpressionType, span: (usize, usize)) -> Self {
        Expression { expression, span }
    }

    pub(crate) fn parse(tokens: &mut TokenStream, level: usize) -> CompilerResult<Expression> {
        if level >= MAX_RECURSION_DEPTH {
            let token = tokens.unshift().unwrap();
            Err(CompilerError {
                kind: ErrorCode::ExceededExpressionRecursion,
                message: format!("Maximale recursiediepte bereikt"),
                span_message: format!("Deze expressie recurseert te ver"),
                span: token.span,
            })
        } else {
            Expression::unary(tokens, level)
        }
    }

    fn unary(tokens: &mut TokenStream, level: usize) -> CompilerResult<Expression> {
        Expression::binary(tokens, level)
    }

    fn binary(tokens: &mut TokenStream, level: usize) -> CompilerResult<Expression> {
        let lhs = if tokens.next_and_after_matches((ARTICLES, ARITH_EXPR)) {
            binary::binary_prefix(tokens, level)
        } else {
            Expression::type_declaration(tokens)
        }?;

        // Note: this may look stupid, but the `if` statement here is necessary
        // to prevent tokens from being mutably borrowed more than once in the
        // block scope of this function.
        let is_negation = if tokens.unshift_if(TokenType::KWNiet).is_some() {
            true
        } else {
            false
        };

        if tokens.next_matches_any(BINARY_OPERATORS) {
            let op = Expression::parse_binary_op(tokens)?;
            let rhs = Expression::parse(tokens, level + 1)?;
            let span = (lhs.span.0, rhs.span.1);

            let expr = Expression {
                expression: Binary(BinaryExpression {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                span,
            };

            if is_negation {
                Ok(Expression {
                    expression: Unary(UnaryExpression {
                        op: UnaryOp::Negation,
                        expr: Box::new(expr),
                    }),
                    span,
                })
            } else {
                Ok(expr)
            }
        } else {
            if is_negation {
                Err(CompilerError {
                    kind: ErrorCode::InvalidExpression,
                    message: format!("Onverwacht {}", TokenType::KWNiet),
                    span_message: format!("Verwachtte geen {} hier", TokenType::KWNiet),
                    span: tokens.unshift().unwrap().span,
                })
            } else {
                Ok(lhs)
            }
        }
    }

    fn type_declaration(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        if tokens.next_matches_any(LITERALS) {
            let token = tokens.unshift().unwrap();
            literal(token)
        } else if tokens.next_matches(TokenType::Identifier) {
            let identifier = tokens.unshift().unwrap();
            Ok(Expression {
                expression: ExpressionType::Identifier(identifier.value.clone()),
                span: identifier.span,
            })
        } else {
            let type_ = parse_type(tokens)?;
            Ok(Expression {
                expression: ExpressionType::Type(type_.0),
                span: type_.1,
            })
        }
    }

    fn parse_binary_op(tokens: &mut TokenStream) -> CompilerResult<BinaryOp> {
        match tokens.unshift().unwrap().type_ {
            TokenType::KWKleiner => {
                tokens.unshift_expect(TokenType::KWIs)?;
                tokens.unshift_expect(TokenType::KWDan)?;
                Ok(BinaryOp::Less)
            }
            TokenType::KWGroter => {
                tokens.unshift_expect(TokenType::KWIs)?;
                tokens.unshift_expect(TokenType::KWDan)?;
                Ok(BinaryOp::Greater)
            }
            TokenType::KWGelijk => {
                tokens.unshift_expect(TokenType::KWIs)?;
                tokens.unshift_expect(TokenType::KWAan)?;
                Ok(BinaryOp::Equal)
            }
            _ => unreachable!(),
        }
    }
}
