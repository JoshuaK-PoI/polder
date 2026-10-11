use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    token::{Span, TokenStream, TokenType},
    unshift::Unshift,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Expression {
    pub(crate) expression: ExpressionType,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum ExpressionType {
    StringExpr(StringExpression),
    Identifier(IdentifierExpresion),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StringExpression {
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct IdentifierExpresion {
    pub(crate) name: String,
}

impl Expression {
    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        Ok(Expression::string(tokens)?)
    }

    fn string(tokens: &mut TokenStream) -> CompilerResult<Expression> {
        if let Some(token) = tokens.peek() {
            match token.type_ {
                TokenType::StringLiteral => {
                    let token = tokens.unshift().unwrap().clone();

                    let text = token.value.replace(&['„', '”', '"'], "");
                    Ok(Expression {
                        expression: ExpressionType::StringExpr(StringExpression { text }),
                        span: token.span,
                    })
                }
                TokenType::Identifier => {
                    let token = tokens.unshift().unwrap().clone();

                    Ok(Expression {
                        expression: ExpressionType::Identifier(IdentifierExpresion {
                            name: token.value,
                        }),
                        span: token.span,
                    })
                }
                _ => Err(CompilerError {
                    error_code: ErrorCode::InvalidExpression,
                    error_message: format!(
                        "Invalid expression: did not expect token of type {}",
                        token.type_
                    ),
                    span_message: String::from(""),
                    span: tokens.unshift().unwrap().span.clone(),
                    help: None,
                    info: None,
                }),
            }
        } else {
            Err(CompilerError {
                error_code: ErrorCode::InvalidExpression,
                error_message: format!(
                    "Invalid expression: did not get a token before end of file"
                ),
                span_message: String::from(""),
                span: 0..0,
                help: None,
                info: None,
            })
        }
    }
}
