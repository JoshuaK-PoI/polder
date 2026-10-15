use serde::Serialize;

use crate::lang::compiler::lexer::token::{KEYWORDS, Token};

use super::super::{
    CompilerError, CompilerResult, ErrorCode,
    iter::unshift::{Unshift, UnshiftExpect},
    lexer::token::{LITERALS, Span, TokenStream, TokenType},
    numeric_literal::parse_numeric_literal,
};

use super::{
    expression::Expression,
    intrinsic::Intrinsics,
    types::{PolderType, parse_type},
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
    pub(crate) variable_type: PolderType,
    pub(crate) value_literal: Option<Token>,
    pub(crate) is_const: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VariableAssignmentStatement {
    pub(crate) identifier: String,
    pub(crate) value: Token,
}

impl Statement {
    pub(crate) fn parse(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        if let Some(token) = tokens.peek() {
            let stmt = match token.type_ {
                TokenType::KWDruk => Statement::intrinsic(Intrinsics::Print, tokens),
                TokenType::KWSchrijf => Statement::intrinsic(Intrinsics::Write, tokens),
                TokenType::KWMaak => Statement::variable_assignment(tokens),
                TokenType::KWLaat => Statement::variable_declaration(tokens),
                TokenType::KWLees => Statement::intrinsic(Intrinsics::Readline, tokens),
                _ => todo!("Statement::parse < {} >", token.type_),
            }?;

            let fullstop = tokens.unshift_expect(TokenType::FullStop);
            if fullstop.is_err() {
                Err(CompilerError {
                    kind: ErrorCode::StatementNotTerminated,
                    message: format!("Een zin eindigt met een punt."),
                    span_message: format!("Ik verwachtte hier het einde van een zin."),
                    span: stmt.1,
                })
            } else {
                Ok(stmt)
            }
        } else {
            Err(CompilerError {
                kind: ErrorCode::NoTokensLeft,
                message: format!("Expected start of statement"),
                span_message: String::from(""),
                span: 0..0,
            })
        }
    }

    pub fn variable_assignment(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWMaak)?;
        tokens.unshift_expect(TokenType::KWVan)?;
        let identifier = tokens.unshift_expect(TokenType::Identifier)?;
        let value = tokens.unshift_expect_any(LITERALS)?;
        let end = value.span.end;

        Ok((
            StatementType::VariableAssignment(VariableAssignmentStatement {
                identifier: identifier.value,
                value: value,
            }),
            start.span.start..end,
        ))
    }

    pub fn variable_declaration(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWLaat)?;
        let identifier = Statement::get_identifier(tokens, &start)?;
        let is_const = tokens.unshift_if(TokenType::KWConst).is_some();
        let is_indefinite_article = tokens.next_matches(TokenType::KWArticleIndef);

        if tokens.next_matches(TokenType::KWZijn) {
            return Err(CompilerError {
                kind: ErrorCode::CannotInferVariableType,
                message: format!("Wat is {} dan?", identifier.value),
                span_message: format!(
                    "Je moet een type of een waarde toewijzen aan {}",
                    identifier.value
                ),
                span: identifier.span.clone(),
            });
        }

        let type_ = parse_type(tokens)?;

        if is_indefinite_article {
            if is_const {
                return Err(CompilerError {
                    kind: ErrorCode::ConstDeclarationMustHaveValue,
                    message: format!("Geen waarde toegewezen aan constante."),
                    span_message: format!("Een constante variabele heeft een waarde nodig."),
                    span: identifier.span.clone(),
                });
            }

            if !tokens.next_matches(TokenType::KWZijn) {
                let token = tokens.unshift();
                return Err(CompilerError {
                    kind: ErrorCode::IndefiniteCannotHaveValue,
                    message: format!("'een {}' kan geen waarde hebben", type_),
                    span_message: format!(
                        "Gebruik '{} {}' als je een waarde opgeeft.",
                        PolderType::article_for(&type_),
                        type_
                    ),
                    span: if token.is_some() {
                        identifier.span.start..token.unwrap().span.end
                    } else {
                        identifier.span.clone()
                    },
                });
            }

            let end = tokens.unshift_expect(TokenType::KWZijn)?;
            return Ok((
                StatementType::VariableDeclaration(VariableDeclarationStatement {
                    identifier: identifier.value,
                    variable_type: type_,
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
                variable_type: type_,
                value_literal: Some(value.unwrap()),
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

        arguments.push(Expression::parse(tokens)?);

        let end = tokens.unshift_expect(intrinsic_name.expected_end_token())?;

        Ok((
            StatementType::Intrinsic(IntrinsicStatement {
                intrinsic: intrinsic_name,
                args: arguments,
            }),
            start.span.start..end.span.end,
        ))
    }

    fn get_identifier(
        tokens: &mut TokenStream,
        start_token: &Token,
    ) -> Result<Token, CompilerError> {
        match tokens.unshift() {
            None => Err(CompilerError {
                kind: ErrorCode::NoTokensLeft,
                message: format!("Naam van de variabele mist"),
                span_message: format!("Geef een naam voor de variabele op."),
                span: start_token.span.clone(),
            }),
            Some(token) => {
                if KEYWORDS.contains_key(&token.value) {
                    Err(CompilerError {
                        kind: ErrorCode::InvalidIdentifier,
                        message: format!("'{}' is een gereserveerd woord.", token.value),
                        span_message: format!("'{}' is een gereserveerd woord.", token.value),
                        span: token.span.clone(),
                    })
                } else if token.type_ != TokenType::Identifier {
                    Err(CompilerError {
                        kind: ErrorCode::InvalidIdentifier,
                        message: format!("Verwacht hier de naam voor de variabele."),
                        span_message: format!(
                            "'{}' is geen geldige naam voor een variabele.",
                            token.value
                        ),
                        span: token.span.clone(),
                    })
                } else {
                    Ok(token.to_owned())
                }
            }
        }
    }
}
