use std::collections::HashMap;

use crate::{
    lang::{
        compiler::parser::{expression::literal::Literal, types::PolderType},
        interpreter::runtime_error::{RuntimeError, RuntimeErrorKind},
    },
    runtime_error,
};

pub(crate) struct Store {
    variables: HashMap<String, Variable>,
}

#[derive(Debug, Clone)]
pub(crate) struct Variable {
    pub(crate) declared_at: (usize, usize),
    pub(crate) type_: PolderType,
    pub(crate) value: Option<Literal>,
    pub(crate) is_const: bool,
}

impl Store {
    pub(crate) fn new() -> Store {
        Store {
            variables: HashMap::new(),
        }
    }

    pub(crate) fn declare(
        &mut self,
        identifier: String,
        variable: Variable,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        if self.variable_defined(&identifier) {
            return Err(runtime_error!(
                range,
                RuntimeErrorKind::VariableAlreadyDefined,
                identifier
            ));
        }

        self.check_type_compatible(&identifier, &variable, None, range)?;

        self.variables.insert(identifier, variable);
        Ok(())
    }

    pub(crate) fn write(
        &mut self,
        identifier: String,
        value: &Literal,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        if !self.variable_defined(&identifier) {
            return Err(runtime_error!(
                range,
                RuntimeErrorKind::UndefinedVariable,
                identifier
            ));
        }

        let variable = self
            .get(&identifier)
            .expect("unreachable (already checked existence");

        self.check_const(&identifier, variable, range)?;
        self.check_type_compatible(&identifier, variable, Some(value), range)?;

        self.variables
            .entry(identifier)
            .and_modify(|variable| variable.value = Some(value.clone()));

        Ok(())
    }

    pub(crate) fn get(&self, identifier: &str) -> Option<&Variable> {
        self.variables.get(identifier)
    }

    pub(crate) fn write_input(
        &mut self,
        identifier: String,
        input: String,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        if !self.variable_defined(&identifier) {
            return Err(runtime_error!(
                range,
                RuntimeErrorKind::UndefinedVariable,
                identifier
            ));
        }

        self.check_const(
            &identifier,
            self.get(&identifier).expect("unreachable"),
            range,
        )?;
        let literal = self.try_marshalling_type(&identifier, input)?;

        self.variables.entry(identifier).and_modify(|variable| {
            if variable.value.is_none() {
                variable.value = Some(literal);
            } else {
                todo!("Typecheck and fill existing variable")
            }
        });

        Ok(())
    }

    fn variable_defined(&self, identifier: &str) -> bool {
        self.variables.contains_key(identifier)
    }

    fn check_type_compatible(
        &self,
        identifier: &str,
        variable: &Variable,
        input: Option<&Literal>,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        let value_to_type_check = input.or(variable.value.as_ref());
        match value_to_type_check {
            // We can't type check an empty value,
            // expect later in the program to handle empty value cases
            None => Ok(()),
            Some(ref value) => {
                if match value {
                    Literal::Boolean(_) => variable.type_ == PolderType::Boolean,
                    Literal::Char(_) => variable.type_ == PolderType::Char,
                    Literal::String(_) => variable.type_ == PolderType::String,
                    Literal::Integer(_) => variable.type_ == PolderType::Integer,
                    Literal::Float(_) => variable.type_ == PolderType::Float,
                } {
                    Ok(())
                } else {
                    Err(runtime_error!(
                        range,
                        RuntimeErrorKind::MismatchedType,
                        "",
                        format!(
                            "{} is een {}, geen {}",
                            identifier,
                            variable.type_,
                            value.get_type()
                        )
                    ))
                }
            }
        }
    }

    fn check_const(
        &self,
        identifier: &str,
        variable: &Variable,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        if variable.is_const {
            Err(runtime_error!(
                range,
                RuntimeErrorKind::VariableIsConst,
                identifier,
                format!(
                    "{} is voorgoed {}",
                    identifier,
                    variable.get_value().as_ref().unwrap()
                )
            ))
        } else {
            Ok(())
        }
    }

    fn try_marshalling_type(
        &self,
        identifier: &str,
        input: String,
    ) -> Result<Literal, RuntimeError> {
        let type_ = &self
            .get(identifier)
            .expect("try_marshalling_type: attempted to get undefined variable")
            .type_;

        match type_ {
            PolderType::Boolean => try_bool(input),
            PolderType::String => Ok(Literal::String(input)),
            PolderType::Integer => try_integer(input),
            PolderType::Float => try_float(input),
            PolderType::Char => try_char(input),
        }
    }
}

fn try_bool(input: String) -> Result<Literal, RuntimeError> {
    match input.as_str() {
        "waar" => Ok(Literal::Boolean(true)),
        "onwaar" => Ok(Literal::Boolean(false)),
        _ => Err(runtime_error!(
            (0, 0),
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte 'waar' of 'onwaar' hier")
        )),
    }
}

// @TODO: Deduplicate this code in future, has overlap with lexer
// @TODO: Runtime errors below should not point to source file in the below cases because they are coming from stdin
// @TODO: Runtime errors below should not abort the program but read from stdin again
fn try_integer(input: String) -> Result<Literal, RuntimeError> {
    match input.parse::<i64>() {
        Ok(int) => Ok(Literal::Integer(int)),
        Err(_) => Err(runtime_error!(
            (0, 0),
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte een getal hier")
        )),
    }
}

fn try_float(input: String) -> Result<Literal, RuntimeError> {
    match input.replace(',', ".").parse::<f64>() {
        Ok(flt) => Ok(Literal::Float(flt)),
        Err(_) => Err(runtime_error!(
            (0, 0),
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Ongeldig karakter in het kommagetal")
        )),
    }
}

fn try_char(input: String) -> Result<Literal, RuntimeError> {
    match input.parse::<char>() {
        Ok(c) => Ok(Literal::Char(c)),
        Err(_) => Err(runtime_error!(
            (0, 0),
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte een enkel karakter hier")
        )),
    }
}

impl Variable {
    pub(crate) fn get_value(&self) -> &Option<Literal> {
        &self.value
    }
}
