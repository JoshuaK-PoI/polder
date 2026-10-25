use std::collections::HashMap;

use log::debug;

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

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StoreValue {
    Literal(Literal),
    Array(Vec<Literal>),
}

impl std::fmt::Display for StoreValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StoreValue::Literal(literal) => literal.to_string(),
                StoreValue::Array(values) => values
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            }
        )
    }
}

impl StoreValue {
    pub(crate) fn get_type(&self) -> String {
        match self {
            StoreValue::Literal(literal) => literal.get_type(),
            StoreValue::Array(elements) => elements
                .into_iter()
                .map(|literal| literal.get_type())
                .collect::<Vec<String>>()
                .join(", "),
        }
    }

    pub(crate) fn literal(
        store_value: StoreValue,
        range: (usize, usize),
        func: fn(value: Literal) -> Literal,
    ) -> Result<StoreValue, RuntimeError> {
        let lit = StoreValue::unwrap_literal(store_value, range)?;
        let res = func(lit);
        Ok(StoreValue::Literal(res))
    }

    pub(crate) fn unwrap_literal(
        store_value: StoreValue,
        range: (usize, usize),
    ) -> Result<Literal, RuntimeError> {
        match store_value {
            StoreValue::Literal(l) => Ok(l),
            StoreValue::Array(_) => Err(runtime_error!(
                range,
                RuntimeErrorKind::CannotOperateOnArray
            )),
        }
    }
}

impl From<&StoreValue> for PolderType {
    fn from(value: &StoreValue) -> Self {
        match value {
            StoreValue::Literal(literal) => literal.into(),
            StoreValue::Array(arr) => PolderType::Array(PolderType::get_array_type(arr)),
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Variable {
    pub(crate) declared_at: (usize, usize),
    pub(crate) type_: PolderType,
    pub(crate) value: Option<StoreValue>,
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

        Store::check_type_compatible(&identifier, &variable, None, range)?;

        self.variables.insert(identifier, variable);
        Ok(())
    }

    pub(crate) fn write(
        &mut self,
        identifier: String,
        value: &StoreValue,
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
        Store::check_type_compatible(&identifier, variable, Some(value), range)?;

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
        let store_value = self.try_marshalling_type(&identifier, input)?;
        let mut modified: Result<(), RuntimeError> = Ok(());

        self.variables
            .entry(identifier.clone())
            .and_modify(|variable| {
                if variable.value.is_some()
                    && Store::check_type_compatible(
                        &identifier,
                        variable,
                        Some(&store_value),
                        range,
                    )
                    .is_err()
                {
                    modified = Err(runtime_error!(
                        range,
                        RuntimeErrorKind::MismatchedType,
                        format!(
                            "Verwachtte een {}, maar zag een {}",
                            variable.type_,
                            store_value.get_type()
                        ),
                        format!(
                            "Verwachtte een {}, maar zag een {}",
                            variable.type_,
                            store_value.get_type()
                        )
                    ))
                }

                variable.value = Some(store_value);
            });

        modified
    }

    fn variable_defined(&self, identifier: &str) -> bool {
        self.variables.contains_key(identifier)
    }

    fn check_type_compatible(
        identifier: &str,
        variable: &Variable,
        input: Option<&StoreValue>,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        let value_to_type_check = input.or(variable.value.as_ref());
        debug!(
            "Value to type check: {:?} against {}",
            value_to_type_check, variable.type_
        );
        match value_to_type_check {
            // We can't type check an empty value,
            // expect later in the program to handle empty value cases
            None => Ok(()),
            Some(value) => match value {
                StoreValue::Literal(literal) => {
                    Store::type_check_literal(identifier, &variable.type_, literal, range)
                }
                StoreValue::Array(elements) => {
                    match &variable.type_ {
                        PolderType::Array(inner_type) => match inner_type {
                            None => Ok(()),
                            Some(type_) => elements.into_iter().try_for_each(|element| {
                                Store::type_check_literal(identifier, type_, element, range)
                                    .map_err(|_| runtime_error!(
                                        range,
                                        RuntimeErrorKind::MismatchedType,
                                        format!("{}:", element),
                                        format!("{} is een {}, {} (van type {}) kan hier niet aan worden toegevoegd.", 
                                            identifier, 
                                            Into::<PolderType>::into(value),
                                            element,
                                            element.get_type()
                                        )
                                    ))
                                }),
                        },
                        _ => Err(runtime_error!(
                            range,
                            RuntimeErrorKind::MismatchedType,
                            format!(
                                "Waarde is een {}, maar het type van {} is {}",
                                Into::<PolderType>::into(value),
                                identifier,
                                variable.type_
                            )
                        )),
                    }
                    // Store::type_check_array(identifier, &variable.type_, elements, range)
                }
            },
        }
    }

    fn type_check_literal(
        identifier: &str,
        literal_type: &PolderType,
        value: &Literal,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        if match value {
            Literal::Boolean(_) => *literal_type == PolderType::Boolean,
            Literal::Char(_) => *literal_type == PolderType::Char,
            Literal::String(_) => *literal_type == PolderType::String,
            Literal::Integer(_) => *literal_type == PolderType::Integer,
            Literal::Float(_) => *literal_type == PolderType::Float,
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
                    literal_type,
                    value.get_type()
                )
            ))
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
    ) -> Result<StoreValue, RuntimeError> {
        let type_ = &self
            .get(identifier)
            .expect("try_marshalling_type: attempted to get undefined variable")
            .type_;

        match type_ {
            PolderType::Boolean => try_bool(input),
            PolderType::String => Ok(StoreValue::Literal(Literal::String(input))),
            PolderType::Integer => try_integer(input),
            PolderType::Float => try_float(input),
            PolderType::Char => try_char(input),
            PolderType::Array(_) => unimplemented!("Marshalling into array type"),
        }
    }
}

fn try_bool(input: String) -> Result<StoreValue, RuntimeError> {
    match input.as_str() {
        "waar" => Ok(StoreValue::Literal(Literal::Boolean(true))),
        "onwaar" => Ok(StoreValue::Literal(Literal::Boolean(false))),
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
fn try_integer(input: String) -> Result<StoreValue, RuntimeError> {
    match input.parse::<i64>() {
        Ok(int) => Ok(StoreValue::Literal(Literal::Integer(int))),
        Err(_) => Err(runtime_error!(
            (0, 0),
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte een getal hier")
        )),
    }
}

fn try_float(input: String) -> Result<StoreValue, RuntimeError> {
    match input.replace(',', ".").parse::<f64>() {
        Ok(flt) => Ok(StoreValue::Literal(Literal::Float(flt))),
        Err(_) => Err(runtime_error!(
            (0, 0),
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Ongeldig karakter in het kommagetal")
        )),
    }
}

fn try_char(input: String) -> Result<StoreValue, RuntimeError> {
    match input.parse::<char>() {
        Ok(c) => Ok(StoreValue::Literal(Literal::Char(c))),
        Err(_) => Err(runtime_error!(
            (0, 0),
            RuntimeErrorKind::MismatchedType,
            input,
            String::from("Verwachtte een enkel karakter hier")
        )),
    }
}

impl Variable {
    pub(crate) fn get_value(&self) -> &Option<StoreValue> {
        &self.value
    }
}
