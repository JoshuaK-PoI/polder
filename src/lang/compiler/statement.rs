use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    expression::Expression,
    intrinsic::Intrinsics,
    token::{Span, TokenStream, TokenType},
    unshift::{Unshift, UnshiftExpect},
};

pub(crate) type SpannedStatement = (StatementType, Span);

#[derive(Debug, Clone)]
pub(crate) enum StatementType {
    Unknown,
    Intrinsic(IntrinsicStatement),
}

impl From<StatementType> for String {
    fn from(value: StatementType) -> Self {
        format!("{:?}", value)
    }
}

impl std::fmt::Display for StatementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from(self.clone()))
    }
}

pub(crate) struct Statement {
    pub(crate) statement: StatementType,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct IntrinsicStatement {
    pub(crate) intrinsic: Intrinsics,
    pub(crate) args: Vec<Expression>,
}

impl Statement {
    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        if let Some(token) = tokens.peek() {
            let stmt = match token.type_ {
                TokenType::Druk => Statement::intrinsic(Intrinsics::Print, tokens),
                _ => todo!("Statement::parse < {} >", token.type_),
            };

            let fullstop = tokens.unshift_expect(TokenType::FullStop);
            if fullstop.is_err() {
                Err(CompilerError {
                    error_code: ErrorCode::StatementNotTerminated,
                    error_message: format!("Expected end of statement"),
                    span_message: format!("Expected a '.' after this"),
                    span: fullstop.err().unwrap().span.clone(),
                    help: None,
                    info: None,
                })
            } else {
                stmt
            }
        } else {
            Err(CompilerError {
                error_code: ErrorCode::NoTokensLeft,
                error_message: format!("Expected start of statement"),
                span_message: String::from(""),
                span: 0..0,
                help: None,
                info: None,
            })
        }
    }

    pub fn intrinsic(
        intrinsic_name: Intrinsics,
        tokens: &mut TokenStream,
    ) -> CompilerResult<SpannedStatement> {
        let start = tokens
            .unshift_expect(intrinsic_name.expected_start_token())?
            .clone();
        let mut arguments = Vec::new();

        while !tokens.next_matches(intrinsic_name.expected_end_token()) {
            arguments.push(Expression::parse(tokens)?);

            if !tokens.next_matches(intrinsic_name.expected_end_token()) {
                return Err(CompilerError {
                    error_code: ErrorCode::UnexpectedToken,
                    error_message: format!("Did not expect token {}", tokens.peek().unwrap().type_),
                    span_message: String::from(""),
                    span: tokens.unshift().unwrap().span.clone(),
                    help: Some(format!(
                        "Expected the end of the statement: {}",
                        intrinsic_name.expected_end_token(),
                    )),
                    info: None,
                });
            }
        }

        let end = tokens
            .unshift_expect(intrinsic_name.expected_end_token())?
            .clone();

        Ok((
            StatementType::Intrinsic(IntrinsicStatement {
                intrinsic: intrinsic_name,
                args: arguments,
            }),
            start.span.start..end.span.end,
        ))
    }
}
