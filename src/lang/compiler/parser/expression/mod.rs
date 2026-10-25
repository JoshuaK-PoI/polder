pub(crate) mod binary;
pub(crate) mod literal;

use crate::lang::compiler::{
    CompilerResult,
    lexer::token::{ARITH_EXPR, ARTICLES, BINARY_OPERATORS, LITERALS, TokenType},
    parser::{
        Parser,
        expression::ExpressionType::{Binary, MethodCall, Unary},
        statement::{BinaryOp, UnaryOp},
        types::PolderType,
    },
};

use literal::{Literal, literal};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Expression {
    pub(crate) expression: ExpressionType,
    pub(crate) span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum ExpressionType {
    ArrayDecl(Vec<Expression>),
    MethodCall(MethodCallExpression),
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct MethodCallExpression {
    pub(crate) method_name: String,
    pub(crate) arguments: Vec<Expression>,
    pub(crate) class: Box<Expression>,
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
                ExpressionType::ArrayDecl(_) => "ArrayDecl".to_string(),
                ExpressionType::MethodCall(_) => "MethodCall".to_string(),
            }
        )
    }
}

impl Expression {
    pub(crate) fn make(expression: ExpressionType, span: (usize, usize)) -> Self {
        Expression { expression, span }
    }
}

impl Parser {
    pub(super) fn expression(&mut self) -> CompilerResult<Expression> {
        self.comparison()
    }

    fn comparison(&mut self) -> CompilerResult<Expression> {
        let lhs = self.binary()?;
        let negate = self.next_is_then_unshift(TokenType::KWNiet);

        if self.next_is_any_of(BINARY_OPERATORS) {
            let token_type = self.next().type_;
            self.expect(TokenType::KWIs)?;
            self.expect(if token_type == TokenType::KWGelijk {
                TokenType::KWAan
            } else {
                TokenType::KWDan
            })?;

            let rhs = self.binary()?;
            let span = (lhs.span.0, rhs.span.1);
            let expression = Expression {
                expression: Binary(BinaryExpression {
                    op: token_type.into(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                span,
            };

            if negate {
                return Ok(Expression {
                    expression: Unary(UnaryExpression {
                        op: UnaryOp::Negation,
                        expr: Box::new(expression),
                    }),
                    span,
                });
            }

            return Ok(expression);
        }

        if negate {
            todo!("Negate without a comparison to make");
        }

        Ok(lhs)
    }

    fn binary(&mut self) -> CompilerResult<Expression> {
        if self.next_is_any_of(ARTICLES) {
            if self.next_nth_is_any_of(2, ARITH_EXPR) {
                self.binary_prefix()
            } else if self.next_nth_is(3, TokenType::KWVan)
                && self.next_nth_is(4, TokenType::Identifier)
            {
                self.function_call()
            } else {
                self.type_declaration()
            }
        } else {
            self.type_declaration()
        }
    }

    fn function_call(&mut self) -> CompilerResult<Expression> {
        let start = self.expect_any_of(ARTICLES)?.span.0;

        let function_name = self.expect(TokenType::Identifier)?.value;
        self.expect(TokenType::KWVan)?;

        let class = self.expression()?;
        let end = class.span.1;

        Ok(Expression {
            expression: MethodCall(MethodCallExpression {
                method_name: function_name,
                arguments: Vec::new(),
                class: Box::new(class),
            }),
            span: (start, end),
        })
    }

    fn type_declaration(&mut self) -> CompilerResult<Expression> {
        if self.next_is_any_of(LITERALS) {
            literal(self.next().clone())
        } else if self.next_is(TokenType::Identifier) {
            let identifier = self.next();
            Ok(Expression {
                expression: ExpressionType::Identifier(identifier.value.clone()),
                span: identifier.span,
            })
        } else {
            let type_ = self.parse_type()?;
            Ok(Expression {
                expression: ExpressionType::Type(type_.0),
                span: type_.1,
            })
        }
    }
}
