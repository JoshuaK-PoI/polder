use serde::Serialize;

use crate::lang::compiler::{
    lexer::token::{KEYWORDS, LITERALS, Token},
    parser::expression::{BinaryExpression, ExpressionType, UnaryExpression, literal::literal},
};

use super::super::{
    CompilerError, CompilerResult, ErrorCode,
    iter::unshift::{Unshift, UnshiftExpect},
    lexer::token::{TokenStream, TokenType},
};

use super::{expression::Expression, intrinsic::Intrinsics, types::PolderType};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum Statement {
    VariableDeclaration(VariableDeclarationStatement),
    VariableAssignment(VariableAssignmentStatement),
    Intrinsic(IntrinsicStatement),
    If(IfStatement),
}

impl From<&Statement> for String {
    fn from(value: &Statement) -> Self {
        format!("{:?}", value)
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from(self))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct IntrinsicStatement {
    pub(crate) intrinsic: Intrinsics,
    pub(crate) args: Vec<Expression>,
    pub(crate) span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct VariableDeclarationStatement {
    pub(crate) identifier: String,
    pub(crate) value_expression: Expression,
    pub(crate) is_const: bool,
    pub(crate) declared_at: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct VariableAssignmentStatement {
    pub(crate) identifier: String,
    pub(crate) expression: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct IfStatement {
    pub(crate) check_expression: Expression,
    pub(crate) then_statement: Box<Statement>,
    pub(crate) else_statement: Box<Option<Statement>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Less,
    Greater,
    Equal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum UnaryOp {
    Increment,
    Decrement,
    Negation,
}

pub(crate) fn parse(tokens: &mut TokenStream, in_then_block: bool) -> CompilerResult<Statement> {
    if let Some(token) = tokens.peek() {
        let span = token.span;

        let stmt = match token.type_ {
            TokenType::KWDruk => intrinsic(Intrinsics::Print, tokens, in_then_block),
            TokenType::KWSchrijf => intrinsic(Intrinsics::Write, tokens, in_then_block),
            TokenType::KWMaak => variable_assignment(tokens, in_then_block),
            TokenType::KWLaat => variable_declaration(tokens, in_then_block),
            TokenType::KWLees => intrinsic(Intrinsics::Readline, tokens, in_then_block),
            TokenType::KWTel => arith_add(tokens, in_then_block),
            TokenType::KWTrek => arith_subtract(tokens, in_then_block),
            TokenType::KWVermenigvuldig => arith_multiply(tokens, in_then_block),
            TokenType::KWDeel => arith_divide(tokens, in_then_block),
            TokenType::KWVerhoog => arith_increment(tokens, in_then_block),
            TokenType::KWVerlaag => arith_decrement(tokens, in_then_block),
            TokenType::KWAls => if_statement(tokens, in_then_block),
            _ => todo!("parse < {} >", token.type_),
        }?;

        if !in_then_block {
            tokens
                .unshift_expect(TokenType::FullStop)
                .map_err(|_| CompilerError {
                    kind: ErrorCode::StatementNotTerminated,
                    message: format!("Een zin eindigt met een punt."),
                    span_message: format!("Deze zin is niet beëindigd."),
                    span
                })?;
        }

        Ok(stmt)
    } else {
        Err(CompilerError {
            kind: ErrorCode::NoTokensLeft,
            message: format!("Expected start of statement"),
            span_message: String::from(""),
            span: (0, 0),
        })
    }
}

pub fn variable_assignment(
    tokens: &mut TokenStream,
    in_then_block: bool,
) -> CompilerResult<Statement> {
    tokens.unshift_expect(TokenType::KWMaak)?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    tokens.unshift_expect(TokenType::KWVan)?;
    let identifier = tokens.unshift_expect(TokenType::Identifier)?.value;
    let expression = Expression::parse(tokens, 0)?;

    Ok(Statement::VariableAssignment(VariableAssignmentStatement {
        identifier,
        expression,
    }))
}

pub fn variable_declaration(
    tokens: &mut TokenStream,
    in_then_block: bool,
) -> CompilerResult<Statement> {
    if in_then_block {
        return Err(CompilerError {
            kind: ErrorCode::InvalidThenStatement,
            message: format!("Hier kan geen variabele gemaakt worden"),
            span_message: format!(
                "Een variabele kan niet in een 'Laat dan...' blok aangemaakt worden."
            ),
            span: tokens.unshift().unwrap().span,
        });
    }
    let start = tokens.unshift_expect(TokenType::KWLaat)?;
    let identifier = get_identifier(tokens, &start)?;
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
            span: identifier.span,
        });
    }

    let expr = Expression::parse(tokens, 0)?;

    let result_expression = match expr.expression {
        ExpressionType::Binary(_) | ExpressionType::Literal(_) => {
            tokens.unshift_expect(TokenType::KWZijn)?;
            Ok(Statement::VariableDeclaration(
                VariableDeclarationStatement {
                    identifier: identifier.value,
                    value_expression: expr,
                    is_const,
                    declared_at: identifier.span
                },
            ))
        }
        ExpressionType::Type(type_) => {
            if is_indefinite_article {
                if is_const {
                    return Err(CompilerError {
                        kind: ErrorCode::ConstDeclarationMustHaveValue,
                        message: format!("Geen waarde toegewezen aan constante."),
                        span_message: format!("Een constante variabele heeft een waarde nodig."),
                        span: identifier.span,
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
                            (identifier.span.0, token.unwrap().span.1)
                        } else {
                            identifier.span
                        },
                    });
                }

                tokens.unshift_expect(TokenType::KWZijn)?;
                return Ok(Statement::VariableDeclaration(
                    VariableDeclarationStatement {
                        identifier: identifier.value,
                        value_expression: expr,
                        is_const,
                        declared_at: identifier.span
                    },
                ));
            }

            let expr = Expression::parse(tokens, 0)?;
            let expr = match expr.expression {
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
            }?;

            tokens.unshift_expect(TokenType::KWZijn)?;
            Ok(Statement::VariableDeclaration(
                VariableDeclarationStatement {
                    identifier: identifier.value,
                    value_expression: expr,
                    is_const,
                    declared_at: identifier.span
                },
            ))
        }
        _expression => Err(CompilerError {
            kind: ErrorCode::IndefiniteCannotHaveValue,
            message: format!("Verwachtte een type hier"),
            span_message: format!("Verwachtte een type hier"),
            span: expr.span,
        }),
    }?;
    Ok(result_expression)
}

pub fn intrinsic(
    intrinsic_name: Intrinsics,
    tokens: &mut TokenStream,
    in_then_block: bool,
) -> CompilerResult<Statement> {
    let start = tokens.unshift_expect(intrinsic_name.expected_start_token())?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    let mut arguments = Vec::new();

    arguments.push(Expression::parse(tokens, 0)?);

    let end = tokens.unshift_expect(intrinsic_name.expected_end_token())?;

    Ok(Statement::Intrinsic(IntrinsicStatement {
        intrinsic: intrinsic_name,
        args: arguments,
        span: (start.span.0, end.span.1),
    }))
}

fn get_identifier(tokens: &mut TokenStream, start_token: &Token) -> Result<Token, CompilerError> {
    match tokens.unshift() {
        None => Err(CompilerError {
            kind: ErrorCode::NoTokensLeft,
            message: format!("Naam van de variabele mist"),
            span_message: format!("Geef een naam voor de variabele op."),
            span: start_token.span,
        }),
        Some(token) => {
            if KEYWORDS.contains_key(&token.value) {
                Err(CompilerError {
                    kind: ErrorCode::InvalidIdentifier,
                    message: format!("'{}' is een gereserveerd woord.", token.value),
                    span_message: format!("'{}' is een gereserveerd woord.", token.value),
                    span: token.span,
                })
            } else if token.type_ != TokenType::Identifier {
                Err(CompilerError {
                    kind: ErrorCode::InvalidIdentifier,
                    message: format!("Verwacht hier de naam voor de variabele."),
                    span_message: format!(
                        "'{}' is geen geldige naam voor een variabele.",
                        token.value
                    ),
                    span: token.span,
                })
            } else {
                Ok(token.to_owned())
            }
        }
    }
}

fn arith_add(tokens: &mut TokenStream, in_then_block: bool) -> Result<Statement, CompilerError> {
    tokens.unshift_expect(TokenType::KWTel)?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    let lhs = Expression::parse(tokens, 0)?;
    tokens.unshift_expect(TokenType::KWBij)?;
    let rhs = tokens.unshift_expect(TokenType::Identifier)?;
    tokens.unshift_expect(TokenType::KWOp)?;

    Ok(variable_assignment_binary(BinaryOp::Add, rhs, lhs))
}

fn arith_subtract(
    tokens: &mut TokenStream,
    in_then_block: bool,
) -> Result<Statement, CompilerError> {
    tokens.unshift_expect(TokenType::KWTrek)?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    let lhs = Expression::parse(tokens, 0)?;
    tokens.unshift_expect(TokenType::KWVan)?;
    let rhs = tokens.unshift_expect(TokenType::Identifier)?;
    tokens.unshift_expect(TokenType::KWAf)?;

    Ok(variable_assignment_binary(BinaryOp::Subtract, rhs, lhs))
}

fn arith_multiply(
    tokens: &mut TokenStream,
    in_then_block: bool,
) -> Result<Statement, CompilerError> {
    tokens.unshift_expect(TokenType::KWVermenigvuldig)?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    let lhs = tokens.unshift_expect(TokenType::Identifier)?;
    tokens.unshift_expect(TokenType::KWMet)?;
    let rhs = Expression::parse(tokens, 0)?;

    Ok(variable_assignment_binary(BinaryOp::Multiply, lhs, rhs))
}

fn arith_divide(tokens: &mut TokenStream, in_then_block: bool) -> Result<Statement, CompilerError> {
    tokens.unshift_expect(TokenType::KWDeel)?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    let lhs = tokens.unshift_expect(TokenType::Identifier)?;
    tokens.unshift_expect(TokenType::KWDoor)?;
    let rhs = Expression::parse(tokens, 0)?;

    Ok(variable_assignment_binary(BinaryOp::Divide, lhs, rhs))
}

fn arith_increment(tokens: &mut TokenStream, in_then_block: bool) -> CompilerResult<Statement> {
    let start = tokens.unshift_expect(TokenType::KWVerhoog)?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    let ident = tokens.unshift_expect(TokenType::Identifier)?;
    let ident_expr = Expression::make(
        ExpressionType::Identifier(ident.value.clone()),
        ident.span,
    );
    Ok(variable_assignment_unary(
        start,
        UnaryOp::Increment,
        ident,
        ident_expr,
    ))
}

fn arith_decrement(tokens: &mut TokenStream, in_then_block: bool) -> CompilerResult<Statement> {
    let start = tokens.unshift_expect(TokenType::KWVerlaag)?;
    if in_then_block {
        tokens.unshift_expect(TokenType::KWDan)?;
    }

    let ident = tokens.unshift_expect(TokenType::Identifier)?;
    let ident_expr = Expression::make(
        ExpressionType::Identifier(ident.value.clone()),
        ident.span,
    );
    Ok(variable_assignment_unary(
        start,
        UnaryOp::Decrement,
        ident,
        ident_expr,
    ))
}

fn if_statement(tokens: &mut TokenStream, in_then_block: bool) -> CompilerResult<Statement> {
    if in_then_block {
        return Err(CompilerError {
            kind: ErrorCode::InvalidThenStatement,
            message: format!("Hier kan niet nogmaals een 'Als' voorkomen"),
            span_message: format!("Een 'als' kan niet het resultaat van een andere 'als' zijn."),
            span: tokens.unshift().unwrap().span,
        });
    }

    tokens.unshift_expect(TokenType::KWAls)?;
    // @TODO: For now, we only support equality/gt/lt expressions
    // When we support AND / OR, we may get more than 1 expression - in that case we will
    // need to loop until we hit the comma.
    let check_expression = Expression::parse(tokens, 0)?;
    let check_expression = create_shorthand_if_statement(tokens, check_expression)?;

    let then_block = parse(tokens, true)?;

    let else_block =
        if tokens.peek_next().is_some() && tokens.peek_next().unwrap().type_ == TokenType::KWZo {
            tokens.unshift_expect(TokenType::FullStop)?;
            tokens.unshift_expect(TokenType::KWZo)?;
            tokens.unshift_expect(TokenType::KWNiet)?;
            tokens.unshift_expect(TokenType::Comma)?;

            // Else-if blocks are also supported by following immediately with another 'Als',
            if tokens.next_matches(TokenType::KWAls) {
                Some(if_statement(tokens, false)?)
            } else {
                Some(parse(tokens, true)?)
            }
        } else {
            None
        };

    Ok(Statement::If(IfStatement {
        check_expression,
        then_statement: Box::new(then_block),
        else_statement: Box::new(else_block),
    }))
}

fn create_shorthand_if_statement(
    tokens: &mut TokenStream,
    check_expression: Expression,
) -> CompilerResult<Expression> {
    let lit = match tokens.peek() {
        Some(token) if token.type_ == TokenType::Comma => {
            tokens.unshift_expect(TokenType::Comma)?;
            Ok(None)
        }
        Some(token) if LITERALS.contains(&token.type_) => {
            let lit = tokens.unshift_expect_any(LITERALS)?;
            tokens.unshift_expect(TokenType::KWIs)?;
            tokens.unshift_expect(TokenType::Comma)?;
            Ok(Some(lit))
        }
        Some(token) => Err(CompilerError {
            kind: ErrorCode::InvalidExpression,
            message: format!("Onverwachte '{}'", token.type_),
            span_message: format!(
                "Verwachtte een literal of komma hier, maar vond '{}'",
                token.type_
            ),
            span: token.span,
        }),
        None => unreachable!(),
    }?;

    if let Some(mut lit) = lit {
        let span = (check_expression.span.0, lit.span.1);

        Ok(Expression {
            expression: ExpressionType::Binary(BinaryExpression {
                op: BinaryOp::Equal,
                lhs: Box::new(check_expression),
                rhs: Box::new(literal(&mut lit)?),
            }),
            span,
        })
    } else {
        Ok(check_expression)
    }
}

fn variable_assignment_unary(
    start: Token,
    op: UnaryOp,
    identifier: Token,
    expr: Expression,
) -> Statement {
    let span = (start.span.0, expr.span.1);
    Statement::VariableAssignment(VariableAssignmentStatement {
        identifier: identifier.value,
        expression: Expression {
            expression: ExpressionType::Unary(UnaryExpression {
                op,
                expr: Box::new(expr),
            }),
            span,
        },
    })
}

fn variable_assignment_binary(op: BinaryOp, target: Token, expr: Expression) -> Statement {
    let span = expr.span;
    Statement::VariableAssignment(VariableAssignmentStatement {
        identifier: target.value.clone(),
        expression: Expression {
            expression: ExpressionType::Binary(BinaryExpression {
                op,
                lhs: Box::new(Expression {
                    expression: ExpressionType::Identifier(target.value),
                    span: target.span,
                }),
                rhs: Box::new(expr),
            }),
            span,
        },
    })
}
