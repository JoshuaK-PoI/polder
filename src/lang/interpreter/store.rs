use std::{collections::HashMap, ops::Range};

use crate::{
    lang::{
        compiler::{
            lexer::token::{KEYWORDS, Token, TokenType},
            parser::{
                statement::{VariableAssignmentStatement, VariableDeclarationStatement},
                types::PolderType,
            },
        },
        interpreter::runtime_error::{RuntimeError, RuntimeErrorKind},
    },
    runtime_error,
};

pub(crate) struct Store {
    variables: HashMap<String, Variable>,
}

#[derive(Debug, Clone)]
pub(crate) struct Variable {
    type_: PolderType,
    identifier: String,
    value: Option<Token>,
    is_const: bool,
}

impl From<&Variable> for String {
    fn from(variable: &Variable) -> Self {
        format!(
            "<{}{}> {} {:<16.16}",
            if variable.is_const { "(const) " } else { "" },
            variable.type_,
            variable.identifier,
            if variable.value.is_some() {
                format!("\"{}\"", variable.value.as_ref().unwrap())
            } else {
                String::from("<uninitialized>")
            },
        )
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl Store {
    pub(crate) fn new() -> Store {
        Store {
            variables: HashMap::new(),
        }
    }

    pub(crate) fn declare(
        &mut self,
        variable: Variable,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        if self.variable_defined(&variable.identifier) {
            return Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::VariableAlreadyDefined,
                variable.identifier
            ));
        }

        self.check_type_compatible(&variable, None)?;

        self.variables.insert(variable.identifier.clone(), variable);
        Ok(())
    }

    pub(crate) fn update(
        &mut self,
        assignment: &VariableAssignmentStatement,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        self.write(&assignment.identifier, &assignment.value, range)
    }

    pub(crate) fn write(
        &mut self,
        identifier: &String,
        value: &Token,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        if !self.variable_defined(identifier) {
            return Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::UndefinedVariable,
                identifier
            ));
        }

        let variable = self
            .get(&identifier)
            .expect("unreachable (already checked existence");

        self.check_const(variable)?;
        self.check_type_compatible(variable, Some(value))?;

        self.variables
            .entry(identifier.clone())
            .and_modify(|variable| variable.value = Some(value.clone()));

        Ok(())
    }

    pub(crate) fn get(&self, identifier: &String) -> Option<&Variable> {
        self.variables.get(identifier)
    }

    pub(crate) fn write_unchecked(
        &mut self,
        identifier: &String,
        input: &String,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        if !self.variable_defined(identifier) {
            return Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::UndefinedVariable,
                identifier
            ));
        }

        self.check_const(self.get(identifier).expect("unreachable"))?;
        let (token_type, token_value) = self.try_marshalling_type(identifier, input)?;

        self.variables
            .entry(identifier.clone())
            .and_modify(|variable| {
                if variable.value.is_none() {
                    variable.value = Some(Token {
                        type_: token_type,
                        value: token_value,
                        span: 0..0,
                    })
                } else {
                    todo!("Typecheck and fill existing variable")
                }
            });

        Ok(())
    }

    fn variable_defined(&self, identifier: &String) -> bool {
        self.variables.contains_key(identifier)
    }

    fn check_type_compatible(
        &self,
        variable: &Variable,
        input: Option<&Token>,
    ) -> Result<(), RuntimeError> {
        let value_to_type_check = input.or(variable.value.as_ref());
        match value_to_type_check {
            // We can't type check an empty value,
            // expect later in the program to handle empty value cases
            None => Ok(()),
            Some(ref value) => {
                if match variable.type_ {
                    PolderType::Boolean => {
                        [TokenType::TrueLiteral, TokenType::FalseLiteral].contains(&value.type_)
                    }
                    PolderType::Char => value.type_ == TokenType::CharLiteral,
                    PolderType::String => value.type_ == TokenType::StringLiteral,
                    PolderType::Integer => value.type_ == TokenType::IntegerLiteral,
                    PolderType::Float => value.type_ == TokenType::FloatLiteral,
                    PolderType::MaybeInteger => value.type_ == TokenType::IntegerLiteral,
                } {
                    Ok(())
                } else {
                    Err(runtime_error!(
                        value.span.clone(),
                        RuntimeErrorKind::MismatchedType,
                        "",
                        format!(
                            "{} is een {}, geen {}",
                            variable.identifier, variable.type_, value.type_
                        )
                    ))
                }
            }
        }
    }

    fn check_const(&self, variable: &Variable) -> Result<(), RuntimeError> {
        if variable.is_const {
            Err(runtime_error!(
                variable.value.as_ref().unwrap().span.clone(), // @TODO: Check - this might fail if the const value is undefined - we don't safeguard against that yet
                RuntimeErrorKind::VariableIsConst,
                variable.identifier,
                format!(
                    "{} is voorgoed {}",
                    variable.identifier,
                    variable.get_value().as_ref().unwrap().value
                )
            ))
        } else {
            Ok(())
        }
    }

    fn try_marshalling_type(
        &self,
        identifier: &String,
        input: &String,
    ) -> Result<(TokenType, String), RuntimeError> {
        let variable_type = &self
            .get(identifier)
            .expect("try_marshalling_type: attempted to get undefined variable")
            .type_;

        match variable_type {
            PolderType::Boolean => try_bool(input),
            PolderType::String => Ok((TokenType::StringLiteral, input.clone())),
            PolderType::Integer => try_integer(input),
            PolderType::MaybeInteger => Err(runtime_error!(
                0..0,
                RuntimeErrorKind::UnsupportedFunction,
                identifier,
                String::from("Marshalling MaybeInteger is not supported")
            )),
            PolderType::Float => try_float(input),
            PolderType::Char => try_char(input),
        }
    }
}

fn try_bool(input: &String) -> Result<(TokenType, String), RuntimeError> {
    if input == "waar" {
        Ok((TokenType::TrueLiteral, input.clone()))
    } else if input == "onwaar" {
        Ok((TokenType::FalseLiteral, input.clone()))
    } else {
        Err(runtime_error!(
            0..0,
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte 'waar' of 'onwaar' hier")
        ))
    }
}

// @TODO: Deduplicate this code in future, has overlap with lexer
// @TODO: Runtime errors below should not point to source file in the below cases because they are coming from stdin
// @TODO: Runtime errors below should not abort the program but read from stdin again
fn try_integer(input: &String) -> Result<(TokenType, String), RuntimeError> {
    if input.chars().all(|c| c.is_ascii_digit()) {
        Ok((TokenType::IntegerLiteral, input.clone()))
    } else {
        Err(runtime_error!(
            0..0,
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte een getal hier")
        ))
    }
}

fn try_float(input: &String) -> Result<(TokenType, String), RuntimeError> {
    if try_integer(input).is_ok() {
        Ok((TokenType::FloatLiteral, input.clone()))
    } else {
        let mut iter = input.chars().into_iter();
        let mut comma_seen = false;
        while let Some(c) = iter.next() {
            if c.is_ascii_digit() {
                continue;
            }

            if c == ',' {
                if comma_seen {
                    return Err(runtime_error!(
                        0..0,
                        RuntimeErrorKind::MismatchedType,
                        input,
                        String::from("Meer dan 1 komma in het kommagetal gevonden")
                    ));
                }
                comma_seen = true;
                continue;
            }

            return Err(runtime_error!(
                0..0,
                RuntimeErrorKind::MismatchedType,
                input,
                String::from("Ongeldig karakter in het kommagetal")
            ));
        }

        Ok((TokenType::FloatLiteral, input.clone()))
    }
}

fn try_char(input: &String) -> Result<(TokenType, String), RuntimeError> {
    if input.trim().len() != 1 {
        Err(runtime_error!(
            0..0,
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte een enkel karakter hier")
        ))
    } else {
        Ok((TokenType::CharLiteral, input.clone()))
    }
}

impl From<&VariableDeclarationStatement> for Variable {
    fn from(declaration: &VariableDeclarationStatement) -> Variable {
        Variable {
            type_: declaration.variable_type.clone(),
            identifier: declaration.identifier.clone(),
            value: declaration.value_literal.clone(),
            is_const: declaration.is_const,
        }
    }
}

impl Variable {
    pub(crate) fn get_value(&self) -> &Option<Token> {
        &self.value
    }
}
