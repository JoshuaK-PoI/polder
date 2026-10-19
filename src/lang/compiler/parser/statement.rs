use serde::Serialize;
use std::ops::Range;

use crate::lang::compiler::{
    lexer::token::{KEYWORDS, Token},
    parser::expression::{
        BinaryExpression, ExpressionType, UnaryExpression
    },
};

use super::super::{
    CompilerError, CompilerResult, ErrorCode,
    iter::unshift::{Unshift, UnshiftExpect},
    lexer::token::{Span, TokenStream, TokenType},
};

use super::{expression::Expression, intrinsic::Intrinsics, types::PolderType};

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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct IntrinsicStatement {
    pub(crate) intrinsic: Intrinsics,
    pub(crate) args: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct VariableDeclarationStatement {
    pub(crate) identifier: String,
    pub(crate) value_expression: Expression,
    pub(crate) is_const: bool,
    pub(crate) span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct VariableAssignmentStatement {
    pub(crate) identifier: String,
    pub(crate) expression: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum UnaryOp {
    Increment,
    Decrement,
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
                TokenType::KWTel => Statement::arith_add(tokens),
                TokenType::KWTrek => Statement::arith_subtract(tokens),
                TokenType::KWVermenigvuldig => Statement::arith_multiply(tokens),
                TokenType::KWDeel => Statement::arith_divide(tokens),
                TokenType::KWVerhoog => Statement::arith_increment(tokens),
                TokenType::KWVerlaag => Statement::arith_decrement(tokens),
                _ => todo!("Statement::parse < {} >", token.type_),
            }?;

            tokens
                .unshift_expect(TokenType::FullStop)
                .map_err(|_| CompilerError {
                    kind: ErrorCode::StatementNotTerminated,
                    message: format!("Een zin eindigt met een punt."),
                    span_message: format!("Ik verwachtte hier het einde van een zin."),
                    span: stmt.1.clone(),
                })?;
            Ok(stmt)
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
        let identifier = tokens.unshift_expect(TokenType::Identifier)?.value;
        let expression = Expression::parse(tokens)?;
        let end = expression.span.end;

        Ok((
            StatementType::VariableAssignment(VariableAssignmentStatement {
                identifier,
                expression,
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

        let expr = Expression::parse(tokens)?;

        let result_expression = match expr.expression {
            ExpressionType::Binary(_) | ExpressionType::Literal(_) => {
                let end = tokens.unshift_expect(TokenType::KWZijn)?;
                Ok((
                    StatementType::VariableDeclaration(VariableDeclarationStatement {
                        identifier: identifier.value,
                        value_expression: expr,
                        is_const,
                        span: start.span.start..end.span.end,
                    }),
                    start.span.start..end.span.end,
                ))
            }
            ExpressionType::Type(type_) => {
                if is_indefinite_article {
                    if is_const {
                        return Err(CompilerError {
                            kind: ErrorCode::ConstDeclarationMustHaveValue,
                            message: format!("Geen waarde toegewezen aan constante."),
                            span_message: format!(
                                "Een constante variabele heeft een waarde nodig."
                            ),
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
                            value_expression: expr,
                            is_const,
                            span: start.span.start..end.span.end,
                        }),
                        start.span.start..end.span.end,
                    ));
                }

                let value = match Expression::parse(tokens) {
                    Ok(expr) => match expr.expression {
                        ExpressionType::Literal(ref lit) => {
                            if type_ != Into::<PolderType>::into(lit) {
                                Err(CompilerError {
                                    kind: ErrorCode::InvalidTypeDeclaration,
                                    message: format!(
                                        "Een {} kan niet worden toegewezen aan een {}",
                                        Into::<PolderType>::into(lit),
                                        type_,
                                    ),
                                    span_message: format!(
                                        "Ik verwachtte hier een {}, maar {} is een {}.",
                                        type_,
                                        identifier.value,
                                        Into::<PolderType>::into(lit)
                                    ),
                                    span: expr.span,
                                })
                            } else {
                                Ok(expr)
                            }
                        }
                        other_type => Err(CompilerError {
                            kind: ErrorCode::InvalidExpression,
                            message: format!(
                                "Verwachtte een literal expression maar zag een {}",
                                other_type
                            ),
                            span_message: "Verwachtte een literal expression".to_string(),
                            span: expr.span,
                        }),
                    },
                    Err(e) => Err(e),
                }?;

                let end = tokens.unshift_expect(TokenType::KWZijn)?;
                Ok((
                    StatementType::VariableDeclaration(VariableDeclarationStatement {
                        identifier: identifier.value,
                        value_expression: value,
                        is_const,
                        span: start.span.start..end.span.end,
                    }),
                    start.span.start..end.span.end,
                ))
            }
            _expression => Err(CompilerError {
                kind: ErrorCode::IndefiniteCannotHaveValue,
                message: format!("Verwachtte een type hier"),
                span_message: format!("Verwachtte een type hier"),
                span: expr.span.clone(),
            }),
        }?;
        Ok(result_expression)
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

    fn arith_add(tokens: &mut TokenStream) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWTel)?;
        let lhs = Expression::parse(tokens)?;
        tokens.unshift_expect(TokenType::KWBij)?;
        let rhs = tokens.unshift_expect(TokenType::Identifier)?;
        let end = tokens.unshift_expect(TokenType::KWOp)?;
        
        Ok(variable_assignment_binary(
            start,
            BinaryOp::Add,
            rhs,
            lhs,
            end.span.end,
        ))
    }

    fn arith_subtract(tokens: &mut TokenStream) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWTrek)?;
        let lhs = Expression::parse(tokens)?;
        tokens.unshift_expect(TokenType::KWVan)?;
        let rhs = tokens.unshift_expect(TokenType::Identifier)?;
        let end = tokens.unshift_expect(TokenType::KWAf)?;
        
        Ok(variable_assignment_binary(
            start,
            BinaryOp::Subtract,
            rhs,
            lhs,
            end.span.end,
        ))
    }

    fn arith_multiply(tokens: &mut TokenStream) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWVermenigvuldig)?;
        let lhs = tokens.unshift_expect(TokenType::Identifier)?;
        tokens.unshift_expect(TokenType::KWMet)?;
        let rhs = Expression::parse(tokens)?;
        let end = rhs.span.end;
        
        Ok(variable_assignment_binary(
            start,
            BinaryOp::Multiply,
            lhs,
            rhs,
            end,
        ))
    }

    fn arith_divide(tokens: &mut TokenStream) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWDeel)?;
        let lhs = tokens.unshift_expect(TokenType::Identifier)?;
        tokens.unshift_expect(TokenType::KWDoor)?;
        let rhs = Expression::parse(tokens)?;
        let end = rhs.span.end;
        
        Ok(variable_assignment_binary(
            start,
            BinaryOp::Divide,
            lhs,
            rhs,
            end,
        ))
    }

    fn arith_increment(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWVerhoog)?;
        let ident = tokens.unshift_expect(TokenType::Identifier)?;
        let ident_expr = Expression::make(
            ExpressionType::Identifier(ident.value.clone()),
            ident.span.clone(),
        );
        Ok(variable_assignment_unary(
            start,
            UnaryOp::Increment,
            ident,
            ident_expr,
        ))
    }

    fn arith_decrement(tokens: &mut TokenStream) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWVerlaag)?;
        let ident = tokens.unshift_expect(TokenType::Identifier)?;
        let ident_expr = Expression::make(
            ExpressionType::Identifier(ident.value.clone()),
            ident.span.clone(),
        );
        Ok(variable_assignment_unary(
            start,
            UnaryOp::Decrement,
            ident,
            ident_expr,
        ))
    }
}

fn variable_assignment_unary(
    start: Token,
    op: UnaryOp,
    identifier: Token,
    expr: Expression,
) -> (StatementType, Range<usize>) {
    let span = start.span.start..expr.span.end;

    (
        StatementType::VariableAssignment(VariableAssignmentStatement {
            identifier: identifier.value,
            expression: Expression {
                expression: ExpressionType::Unary(UnaryExpression {
                    op,
                    expr: Box::new(expr),
                }),
                span: span.clone(),
            },
        }),
        span,
    )
}

fn variable_assignment_binary(
    start: Token,
    op: BinaryOp,
    target: Token,
    expr: Expression,
    end: usize,
) -> (StatementType, Range<usize>) {
    let span = expr.span.clone();
    (
        StatementType::VariableAssignment(VariableAssignmentStatement {
            identifier: target.value.clone(),
            expression: Expression {
                expression: ExpressionType::Binary(BinaryExpression {
                    op,
                    lhs: Box::new(Expression {
                        expression: ExpressionType::Identifier(target.value),
                        span: target.span.clone(),
                    }),
                    rhs: Box::new(expr),
                }),
                span,
            },
        }),
        start.span.start..end,
    )
}
