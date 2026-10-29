use serde::Serialize;

use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    lexer::token::{Token, TokenType},
    parser::types::PolderType,
};

use super::{Expression, ExpressionType};

#[derive(Debug, Clone, PartialEq, Serialize, PartialOrd)]
pub(crate) enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Char(char),
    Boolean(bool),
    Vec(Vec<Literal>),
}

impl std::ops::Add for Literal {
    type Output = Literal;
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Literal::Integer(int) => match rhs {
                Literal::Integer(rhs_int) => Literal::Integer(int + rhs_int),
                Literal::Float(rhs_flt) => Literal::Float(int as f64 + rhs_flt),
                _ => todo!(),
            },
            Literal::Float(flt) => match rhs {
                Literal::Float(rhs_flt) => Literal::Float(flt + rhs_flt),
                Literal::Integer(rhs_int) => Literal::Float(flt + rhs_int as f64),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

impl std::ops::Sub for Literal {
    type Output = Literal;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Literal::Integer(int) => match rhs {
                Literal::Integer(rhs_int) => Literal::Integer(int - rhs_int),
                Literal::Float(rhs_flt) => Literal::Float(int as f64 - rhs_flt),
                _ => todo!(),
            },
            Literal::Float(flt) => match rhs {
                Literal::Float(rhs_flt) => Literal::Float(flt - rhs_flt),
                Literal::Integer(rhs_int) => Literal::Float(flt - rhs_int as f64),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

impl std::ops::Mul for Literal {
    type Output = Literal;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Literal::Integer(int) => match rhs {
                Literal::Integer(rhs_int) => Literal::Integer(int * rhs_int),
                Literal::Float(rhs_flt) => Literal::Float(int as f64 * rhs_flt),
                _ => todo!(),
            },
            Literal::Float(flt) => match rhs {
                Literal::Float(rhs_flt) => Literal::Float(flt * rhs_flt),
                Literal::Integer(rhs_int) => Literal::Float(flt * rhs_int as f64),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

impl std::ops::Div for Literal {
    type Output = Literal;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Literal::Integer(int) => match rhs {
                Literal::Integer(rhs_int) => Literal::Integer(int / rhs_int),
                Literal::Float(rhs_flt) => Literal::Float(int as f64 / rhs_flt),
                _ => todo!(),
            },
            Literal::Float(flt) => match rhs {
                Literal::Float(rhs_flt) => Literal::Float(flt / rhs_flt),
                Literal::Integer(rhs_int) => Literal::Float(flt / rhs_int as f64),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

impl std::ops::Rem for Literal {
    type Output = Literal;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            Literal::Integer(int) => match rhs {
                Literal::Integer(rhs_int) => Literal::Integer(int % rhs_int),
                Literal::Float(rhs_flt) => Literal::Float(int as f64 % rhs_flt),
                _ => todo!(),
            },
            Literal::Float(flt) => match rhs {
                Literal::Float(rhs_flt) => Literal::Float(flt % rhs_flt),
                Literal::Integer(rhs_int) => Literal::Float(flt % rhs_int as f64),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

impl std::ops::Not for Literal {
    type Output = Literal;
    fn not(self) -> Self::Output {
        match self {
            Literal::Boolean(b) => Literal::Boolean(!b),
            _ => unreachable!(),
        }
    }
}
impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Literal::String(s) => s.to_string(),
                Literal::Integer(i) => i
                    .to_string()
                    .as_bytes()
                    .rchunks(3)
                    .rev()
                    .map(std::str::from_utf8)
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap()
                    .join("."),
                Literal::Float(f) => f.to_string().replace('.', ","),
                Literal::Boolean(b) => (if *b { "waar" } else { "onwaar" }).to_string(),
                Literal::Char(c) => c.to_string(),
                Literal::Vec(vec) => vec
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            }
        )
    }
}

impl Into<PolderType> for &Literal {
    fn into(self) -> PolderType {
        match self {
            Literal::Boolean(_) => PolderType::Boolean,
            Literal::Char(_) => PolderType::Char,
            Literal::Float(_) => PolderType::Float,
            Literal::Integer(_) => PolderType::Integer,
            Literal::String(_) => PolderType::String,
            Literal::Vec(v) => PolderType::Array(if v.get(0).is_some() {
                Some(Box::new(v.get(0).unwrap().into()))
            } else {
                None
            }),
        }
    }
}

impl Literal {
    pub(crate) fn get_type(&self) -> String {
        match self {
            Literal::String(_) => "tekst".to_string(),
            Literal::Integer(_) => "getal".to_string(),
            Literal::Float(_) => "kommagetal".to_string(),
            Literal::Boolean(_) => "waarheid".to_string(),
            Literal::Char(_) => "letter".to_string(),
            Literal::Vec(v) => {
                format!(
                    "{}{}",
                    "lijst van".to_string(),
                    if v.get(0).is_some() {
                        v.get(0).unwrap().get_type()
                    } else {
                        "<onbekend>".to_string()
                    }
                )
            }
        }
    }
}

pub(crate) fn literal(token: Token) -> CompilerResult<Expression> {
    let span = token.span;
    Ok(Expression {
        expression: match token.type_ {
            TokenType::Identifier => ExpressionType::Identifier(token.value.clone()),
            TokenType::StringLiteral => {
                ExpressionType::Literal(Literal::String(token.value.replace(&['„', '”', '"'], "")))
            }
            TokenType::IntegerLiteral => ExpressionType::Literal(Literal::Integer(
                token.value.parse::<i64>().map_err(|err| CompilerError {
                    kind: ErrorCode::ParseIntError,
                    message: format!("Could not parse value to int: {}", err),
                    span_message: format!("Cannot parse this to int"),
                    span,
                })?,
            )),
            TokenType::FloatLiteral => ExpressionType::Literal(Literal::Float(
                token
                    .value
                    .replace(',', ".")
                    .parse::<f64>()
                    .map_err(|err| CompilerError {
                        kind: ErrorCode::ParseIntError,
                        message: format!("Could not parse value to float: {}", err),
                        span_message: format!("Cannot parse this to float"),
                        span,
                    })?,
            )),
            TokenType::TrueLiteral | TokenType::FalseLiteral => {
                ExpressionType::Literal(Literal::Boolean(token.type_ == TokenType::TrueLiteral))
            }
            TokenType::CharLiteral => ExpressionType::Literal(Literal::Char(
                token.value.parse::<char>().map_err(|err| CompilerError {
                    kind: ErrorCode::ParseIntError,
                    message: format!("Could not parse value to char: {}", err),
                    span_message: format!("Cannot parse this to char"),
                    span,
                })?,
            )),
            type_ => Err(CompilerError {
                kind: ErrorCode::InvalidExpression,
                message: format!("Invalid expression: did not expect token of type {}", type_),
                span_message: String::from(""),
                span,
            })?,
        },
        span,
    })
}
