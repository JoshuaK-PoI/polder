pub(crate) mod arithmetic;
pub(crate) mod literal;

use crate::lang::compiler::{
    CompilerResult,
    iter::unshift::{Unshift, UnshiftExpect},
    lexer::token::{ARITH_EXPR, ARTICLES, LITERALS, Span, TokenStream, TokenType},
    parser::{
        statement::ArithmeticOp,
        types::{PolderType, parse_type},
    },
};

use arithmetic::arithmetic;
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
    Arithmetic(ArithmeticExpression),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ArithmeticExpression {
    op: ArithmeticOp,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
}

impl std::fmt::Display for ExpressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ExpressionType::Arithmetic(_) => "Arithmetic".to_string(),
                ExpressionType::Literal(_) => "Literal".to_string(),
                ExpressionType::Identifier(_) => "Identifier".to_string(),
                ExpressionType::Type(_) => "Type".to_string(),
            }
        )
    }
}

impl Expression {
    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        Ok(Expression::arithmetic(tokens)?)
    }

    fn arithmetic(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        if tokens.next_and_after_matches((ARTICLES, ARITH_EXPR)) {
            arithmetic(tokens)
        } else {
            Ok(Expression::type_declaration(tokens)?)
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
