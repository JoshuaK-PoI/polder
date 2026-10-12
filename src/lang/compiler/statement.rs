use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    expression::Expression,
    intrinsic::Intrinsics,
    token::{Span, TokenStream, TokenType},
    types::{PolderType, parse_type},
    unshift::{Unshift, UnshiftExpect},
};

pub(crate) type SpannedStatement = (StatementType, Span);

#[derive(Debug, Clone, Serialize)]
pub(crate) enum StatementType {
    VariableDeclaration(VariableDeclarationStatement),
    Intrinsic(IntrinsicStatement),
}

impl From<&StatementType> for String {
    fn from(value: &StatementType) -> Self {
        format!("{:?}", value)
    }
}

impl std::fmt::Display for StatementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from(self))
    }
}

pub(crate) struct Statement {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct IntrinsicStatement {
    pub(crate) intrinsic: Intrinsics,
    pub(crate) args: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VariableDeclarationStatement {
    pub(crate) identifier: String,
    pub(crate) variable_type: Option<PolderType>,
    pub(crate) value_literal: Option<String>,
}

impl Statement {
    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        if let Some(token) = tokens.peek() {
            let stmt = match token.type_ {
                TokenType::KWDruk => Statement::intrinsic(Intrinsics::Print, tokens),
                TokenType::KWLaat => Statement::variable_declaration(tokens),
                _ => todo!("Statement::parse < {} >", token.type_),
            }?;

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
                Ok(stmt)
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

    pub fn variable_declaration(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWLaat)?;
        let identifier = tokens.unshift_expect(TokenType::Identifier)?;
        let type_ = parse_type(tokens)?;

        // @TODO: Const assignment
        // @TODO: Handling of variables without a value
        let value = tokens.unshift_expect_any(&[
            TokenType::StringLiteral,
            TokenType::FloatLiteral,
            TokenType::IntegerLiteral,
            TokenType::TrueLiteral,
            TokenType::FalseLiteral,
            TokenType::CharLiteral,
        ])?;

        let end = tokens.unshift_expect(TokenType::KWZijn)?;

        Ok((
            StatementType::VariableDeclaration(VariableDeclarationStatement {
                identifier: identifier.value,
                variable_type: type_,
                value_literal: Some(value.value),
            }),
            start.span.start..end.span.end,
        ))
    }

    pub fn intrinsic(
        intrinsic_name: Intrinsics,
        tokens: &mut TokenStream,
    ) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(intrinsic_name.expected_start_token())?;
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

        let end = tokens.unshift_expect(intrinsic_name.expected_end_token())?;

        Ok((
            StatementType::Intrinsic(IntrinsicStatement {
                intrinsic: intrinsic_name,
                args: arguments,
            }),
            start.span.start..end.span.end,
        ))
    }
}
