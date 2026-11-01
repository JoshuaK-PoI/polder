pub(crate) mod binary;
pub(crate) mod literal;

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode, Span,
    lexer::token::{ARITH_EXPR, ARTICLES, BINARY_OPERATORS, LITERALS, Token, TokenType},
    parser::{
        Parser,
        statement::{BinaryOp, UnaryOp},
        types::PolderType,
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
    ArrayDecl(Vec<Expression>),
    MethodCall(MethodCallExpression),
    Literal(Literal),
    Identifier(Token),
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
    pub(crate) fn make(expression: ExpressionType, span: Span) -> Self {
        Expression { expression, span }
    }
}

impl Parser {
    pub(super) fn expression(&mut self) -> CompilerResult<Expression> {
        self.array_decl()
    }

    fn array_decl(&mut self) -> CompilerResult<Expression> {
        let elem = self.comparison()?;
        if self.next_is_any_of(&[TokenType::Comma, TokenType::KWEn]) {
            let mut elem_vec = vec![elem];
            loop {
                if self.next_is_then_unshift(TokenType::KWEn) {
                    elem_vec.push(self.comparison()?);
                    break;
                }

                self.expect(TokenType::Comma)?;
                elem_vec.push(self.comparison()?);

                if !self.next_is_any_of(&[TokenType::Comma, TokenType::KWEn]) {
                    return Err(CompilerError::new(
                        ErrorCode::InvalidExpression,
                        format!(
                            "Verwachtte een komma of 'en', maar zag '{}'",
                            self.peek().unwrap().value
                        ),
                        self.peek().unwrap().span,
                    ));
                }
            }

            let span = (
                elem_vec.first().unwrap().span.0,
                elem_vec.last().unwrap().span.1,
            );

            Ok(Expression {
                expression: ExpressionType::ArrayDecl(elem_vec),
                span,
            })
        } else {
            Ok(elem)
        }
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
                expression: ExpressionType::Binary(BinaryExpression {
                    op: token_type.into(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                span,
            };

            if negate {
                return Ok(Expression {
                    expression: ExpressionType::Unary(UnaryExpression {
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
        // TODO: Now that types are not part of the expression parser anymore,
        // it should be possible to clean up and untangle a bunch of the very specific token sequences down here
        if self.next_is_any_of(ARTICLES) {
            if self.next_nth_is_any_of(2, ARITH_EXPR) {
                self.binary_prefix()
            } else if self.next_nth_is_any_of(3, &[TokenType::KWVan, TokenType::KWOp])
                && self.next_nth_is_any_of(4, &[TokenType::Identifier, TokenType::KWPlek])
            {
                self.function_call()
            } else {
                self.literal()
            }
        } else {
            let expr = self.literal()?;

            if self.next_is_any_of(&[TokenType::KWPlus, TokenType::KWMin]) {
                let op = match self.next().type_ {
                    TokenType::KWPlus => BinaryOp::Add,
                    TokenType::KWMin => BinaryOp::Subtract,
                    _ => unreachable!(),
                };

                let rhs = self.expression()?;
                let start = expr.span.0;
                let end = rhs.span.1;
                Ok(Expression {
                    expression: ExpressionType::Binary(BinaryExpression {
                        op,
                        lhs: Box::new(expr),
                        rhs: Box::new(rhs),
                    }),
                    span: (start, end),
                })
            } else {
                Ok(expr)
            }
        }
    }

    fn function_call(&mut self) -> CompilerResult<Expression> {
        let start = self.expect_any_of(ARTICLES)?.span.0;

        let function_name = self.expect(TokenType::Identifier)?.value;
        let mut arguments = Vec::new();

        if self.next_is_then_unshift(TokenType::KWOp) {
            // TODO:
            // For now only supports `element op plek`
            self.expect(TokenType::KWPlek)?;
            arguments.push(self.expression()?);
        }

        self.expect(TokenType::KWVan)?;

        let class = self.expression()?;
        let end = class.span.1;

        Ok(Expression {
            expression:ExpressionType::MethodCall(MethodCallExpression {
                method_name: function_name,
                arguments,
                class: Box::new(class),
            }),
            span: (start, end),
        })
    }

    fn literal(&mut self) -> CompilerResult<Expression> {
        if self.next_is_any_of(LITERALS) {
            literal(self.next().clone())
        } else {
            self.identifier()
        }
    }

    fn identifier(&mut self) -> CompilerResult<Expression> {
        if self.next_is(TokenType::EOF) {
            return Err(CompilerError::new(
                ErrorCode::NoTokensLeft,
                format!("Verwachtte een expressie, maar vond het einde van het bestand"),
                self.tokens.last().unwrap().span,
            ));
        }

        let identifier = self.get_identifier()?;
        let span = identifier.span;
        Ok(Expression {
            expression: ExpressionType::Identifier(identifier),
            span,
        })
    }
}
