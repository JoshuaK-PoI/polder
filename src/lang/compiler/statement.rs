use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    expression::Expression,
    intrinsic::Intrinsics,
    numeric_literal::parse_numeric_literal,
    token::{LITERALS, Span, TokenStream, TokenType},
    types::{PolderType, parse_type},
    unshift::{Unshift, UnshiftExpect},
};

pub(crate) type SpannedStatement = (StatementType, Span);

#[derive(Debug, Clone, Serialize)]
pub(crate) enum StatementType {
    VariableDeclaration(VariableDeclarationStatement),
    VariableAssignment(VariableAssignmentStatement),
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
    pub(crate) is_const: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VariableAssignmentStatement {
    pub(crate) identifier: String,
    pub(crate) value_literal: String,
}

impl Statement {
    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        if let Some(token) = tokens.peek() {
            let stmt = match token.type_ {
                TokenType::KWDruk => Statement::intrinsic(Intrinsics::Print, tokens),
                TokenType::KWMaak => Statement::variable_assignment(tokens),
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

    pub fn variable_assignment(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWMaak)?;
        tokens.unshift_expect(TokenType::KWVan)?;
        let identifier = tokens.unshift_expect(TokenType::Identifier)?;
        let value = tokens.unshift_expect_any(LITERALS)?;
        Ok((
            StatementType::VariableAssignment(VariableAssignmentStatement {
                identifier: identifier.value,
                value_literal: value.value,
            }),
            start.span.start..value.span.end,
        ))
    }

    pub fn variable_declaration(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWLaat)?;
        let identifier = tokens.unshift_expect(TokenType::Identifier)?;
        let is_const = tokens.unshift_if(TokenType::KWConst).is_some();
        let is_indefinite_article = tokens.next_matches(TokenType::KWArticleIndef);
        let type_ = parse_type(tokens)?;

        if is_indefinite_article {
            if !tokens.next_matches(TokenType::KWZijn) {
                let token = tokens.unshift();
                return Err(CompilerError {
                    error_code: ErrorCode::IndefiniteCannotHaveValue,
                    error_message: format!("Indefinite article cannot have a value."),
                    span_message: format!(
                        "Expected {}, got {}",
                        TokenType::KWZijn,
                        if token.is_some() {
                            token.as_ref().unwrap().value.clone()
                        } else {
                            "nothing".to_string()
                        }
                    ),
                    span: if token.is_some() {
                        token.unwrap().span.clone()
                    } else {
                        0..0
                    },
                    help: None,
                    info: None,
                });
            }

            let end = tokens.unshift_expect(TokenType::KWZijn)?;
            return Ok((
                StatementType::VariableDeclaration(VariableDeclarationStatement {
                    identifier: identifier.value,
                    variable_type: Some(type_),
                    value_literal: None,
                    is_const,
                }),
                start.span.start..end.span.end,
            ));
        }

        let mut value = tokens.unshift_expect_any(LITERALS);

        if value.is_err() {
            // We might be parsing a number literal
            // Check we have the numeric type set
            // Otherwise return the error immediately
            if ![PolderType::Integer, PolderType::MaybeInteger].contains(&type_) {
                return Err(value.err().unwrap());
            }

            let token = tokens.unshift_expect(TokenType::Identifier)?;
            value = parse_numeric_literal(token.value, token.span);

            // If we then still can't parse it, it is probably invalid
            if value.is_err() {
                return Err(value.err().unwrap());
            }
        }

        let end = tokens.unshift_expect(TokenType::KWZijn)?;

        Ok((
            StatementType::VariableDeclaration(VariableDeclarationStatement {
                identifier: identifier.value,
                variable_type: Some(type_),
                value_literal: Some(value.unwrap().value),
                is_const,
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
