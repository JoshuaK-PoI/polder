pub(crate) mod binary;
pub(crate) mod literal;
pub(crate) mod unary;

use std::ops::Range;

use crate::lang::compiler::{
    CompilerResult,
    iter::unshift::{Unshift, UnshiftExpect},
    lexer::token::{ARITH_EXPR, ARTICLES, LITERALS, Span, TokenStream, TokenType},
    parser::{
        statement::{BinaryOp, UnaryOp},
        types::{PolderType, parse_type},
    },
};

use literal::{Literal, literal};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Expression {
    pub(crate) expression: ExpressionType,
    pub(crate) span: Span,
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
                ExpressionType::Binary(_) => "Unary".to_string(),
                ExpressionType::Unary(_) => "Binary".to_string(),
                ExpressionType::Literal(_) => "Literal".to_string(),
                ExpressionType::Identifier(_) => "Identifier".to_string(),
                ExpressionType::Type(_) => "Type".to_string(),
            }
        )
    }
}

impl Expression {
    pub(crate) fn make(expression: ExpressionType, span: Range<usize>) -> Self {
        Expression { expression, span }
    }

    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        Expression::unary(tokens)
    }

    fn unary(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        Expression::binary(tokens)
    }

    fn binary(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        if tokens.next_and_after_matches((ARTICLES, ARITH_EXPR)) {
            binary::binary(tokens)
        } else {
            Expression::type_declaration(tokens)
        }
    }

    fn type_declaration(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        if tokens.next_matches_any(LITERALS) {
            literal(tokens)
        } else if tokens.next_matches(TokenType::Identifier) {
            let identifier = tokens.unshift().unwrap();
            Ok(Expression {
                expression: ExpressionType::Identifier(identifier.value.clone()),
                span: identifier.span.clone(),
            })
        } else {
            let type_ = parse_type(tokens)?;
            Ok(Expression {
                expression: ExpressionType::Type(type_.0),
                span: type_.1,
            })
        }
    }
}
