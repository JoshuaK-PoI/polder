use serde::Serialize;
use std::ops::Range;

use crate::lang::compiler::{
    lexer::token::{KEYWORDS, Token},
    parser::expression::{BinaryExpression, ExpressionType, UnaryExpression},
};

use super::super::{
    CompilerError, CompilerResult, ErrorCode,
    iter::unshift::{Unshift, UnshiftExpect},
    lexer::token::{Span, TokenStream, TokenType},
};

use super::{expression::Expression, intrinsic::Intrinsics, types::PolderType};

pub(crate) type SpannedStatement = (StatementType, Span);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum StatementType {
    VariableDeclaration(VariableDeclarationStatement),
    VariableAssignment(VariableAssignmentStatement),
    Intrinsic(IntrinsicStatement),
    If(IfStatement),
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct IfStatement {
    pub(crate) check_expression: Expression,
    pub(crate) then_statement: Box<SpannedStatement>,
    pub(crate) else_statement: Box<Option<SpannedStatement>>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Less,
    Greater,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum UnaryOp {
    Increment,
    Decrement,
}

impl Statement {
    pub(crate) fn parse(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> CompilerResult<SpannedStatement> {
        if let Some(token) = tokens.peek() {
            let stmt = match token.type_ {
                TokenType::KWDruk => Statement::intrinsic(Intrinsics::Print, tokens, in_then_block),
                TokenType::KWSchrijf => {
                    Statement::intrinsic(Intrinsics::Write, tokens, in_then_block)
                }
                TokenType::KWMaak => Statement::variable_assignment(tokens, in_then_block),
                TokenType::KWLaat => Statement::variable_declaration(tokens, in_then_block),
                TokenType::KWLees => {
                    Statement::intrinsic(Intrinsics::Readline, tokens, in_then_block)
                }
                TokenType::KWTel => Statement::arith_add(tokens, in_then_block),
                TokenType::KWTrek => Statement::arith_subtract(tokens, in_then_block),
                TokenType::KWVermenigvuldig => Statement::arith_multiply(tokens, in_then_block),
                TokenType::KWDeel => Statement::arith_divide(tokens, in_then_block),
                TokenType::KWVerhoog => Statement::arith_increment(tokens, in_then_block),
                TokenType::KWVerlaag => Statement::arith_decrement(tokens, in_then_block),
                TokenType::KWAls => Statement::if_statement(tokens, in_then_block),
                _ => todo!("Statement::parse < {} >", token.type_),
            }?;

            if !in_then_block {
                tokens
                    .unshift_expect(TokenType::FullStop)
                    .map_err(|_| CompilerError {
                        kind: ErrorCode::StatementNotTerminated,
                        message: format!("Een zin eindigt met een punt."),
                        span_message: format!("Ik verwachtte hier het einde van een zin."),
                        span: stmt.1.clone(),
                    })?;
            }

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

    pub fn variable_assignment(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWMaak)?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

        tokens.unshift_expect(TokenType::KWVan)?;
        let identifier = tokens.unshift_expect(TokenType::Identifier)?.value;
        let expression = Expression::parse(tokens, 0)?;
        let end = expression.span.end;

        Ok((
            StatementType::VariableAssignment(VariableAssignmentStatement {
                identifier,
                expression,
            }),
            start.span.start..end,
        ))
    }

    pub fn variable_declaration(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> CompilerResult<SpannedStatement> {
        if in_then_block {
            return Err(CompilerError {
                kind: ErrorCode::InvalidThenStatement,
                message: format!("Hier kan geen variabele gemaakt worden"),
                span_message: format!(
                    "Een variabele kan niet in een 'Laat dan...' blok aangemaakt worden."
                ),
                span: tokens.unshift().unwrap().span.clone(),
            });
        }
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

        let expr = Expression::parse(tokens, 0)?;

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

                let value = match Expression::parse(tokens, 0) {
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
        in_then_block: bool,
    ) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(intrinsic_name.expected_start_token())?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

        let mut arguments = Vec::new();

        arguments.push(Expression::parse(tokens, 0)?);

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

    fn arith_add(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWTel)?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

        let lhs = Expression::parse(tokens, 0)?;
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

    fn arith_subtract(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWTrek)?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

        let lhs = Expression::parse(tokens, 0)?;
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

    fn arith_multiply(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWVermenigvuldig)?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

        let lhs = tokens.unshift_expect(TokenType::Identifier)?;
        tokens.unshift_expect(TokenType::KWMet)?;
        let rhs = Expression::parse(tokens, 0)?;
        let end = rhs.span.end;

        Ok(variable_assignment_binary(
            start,
            BinaryOp::Multiply,
            lhs,
            rhs,
            end,
        ))
    }

    fn arith_divide(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> Result<SpannedStatement, CompilerError> {
        let start = tokens.unshift_expect(TokenType::KWDeel)?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

        let lhs = tokens.unshift_expect(TokenType::Identifier)?;
        tokens.unshift_expect(TokenType::KWDoor)?;
        let rhs = Expression::parse(tokens, 0)?;
        let end = rhs.span.end;

        Ok(variable_assignment_binary(
            start,
            BinaryOp::Divide,
            lhs,
            rhs,
            end,
        ))
    }

    fn arith_increment(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWVerhoog)?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

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

    fn arith_decrement(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> CompilerResult<SpannedStatement> {
        let start = tokens.unshift_expect(TokenType::KWVerlaag)?;
        if in_then_block {
            tokens.unshift_expect(TokenType::KWDan)?;
        }

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

    fn if_statement(
        tokens: &mut TokenStream,
        in_then_block: bool,
    ) -> CompilerResult<SpannedStatement> {
        if in_then_block {
            return Err(CompilerError {
                kind: ErrorCode::InvalidThenStatement,
                message: format!("Hier kan niet nogmaals een 'Als' voorkomen"),
                span_message: format!(
                    "Een 'als' kan niet het resultaat van een andere 'als' zijn."
                ),
                span: tokens.unshift().unwrap().span.clone(),
            });
        }

        let start = tokens.unshift_expect(TokenType::KWAls)?;
        let check_expression = Expression::parse(tokens, 0)?;
        // @TODO: For now, we only support equality/gt/lt expressions
        // When we support AND / OR, we may get more than 1 expression - in that case we will
        // need to loop until we hit the comma.
        tokens.unshift_expect(TokenType::Comma)?;

        let then_block = Statement::parse(tokens, true)?;
        
        let else_block = if tokens.peek_next().is_some() && tokens.peek_next().unwrap().type_ == TokenType::KWZo {
            tokens.unshift_expect(TokenType::FullStop)?;
            tokens.unshift_expect(TokenType::KWZo)?;
            tokens.unshift_expect(TokenType::KWNiet)?;
            tokens.unshift_expect(TokenType::Comma)?;
            Some(Statement::parse(tokens, true)?)
        } else { None };

        let span = start.span.start..then_block.1.end;

        Ok((
            StatementType::If(IfStatement {
                check_expression,
                then_statement: Box::new(then_block),
                else_statement: Box::new(else_block),
            }),
            span,
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
