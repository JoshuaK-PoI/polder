use std::collections::HashMap;

use crate::{
    lang::{
        compiler::parser::{expression::literal::Literal, types::PolderType},
        interpreter::runtime_error::{RuntimeError, RuntimeErrorKind},
    },
    runtime_error,
};

pub(crate) struct Store {
    variables: HashMap<usize, HashMap<String, Variable>>,
    scope: usize,
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
            StoreValue::Array(arr) => PolderType::Array(Some(PolderType::get_array_type(arr))),
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Variable {
    pub(crate) declared_at: (usize, usize),
    pub(crate) type_: PolderType,
    pub(crate) value: Option<(StoreValue, (usize, usize))>,
    pub(crate) is_const: bool,
}

impl Store {
    pub(crate) fn new() -> Store {
        let mut store = Store {
            variables: HashMap::new(),
            scope: 0,
        };

        store.variables.insert(store.scope, HashMap::new());
        store
    }

    pub(crate) fn up_scope(&mut self) {
        self.scope += 1;
        self.variables.insert(self.scope, HashMap::new());
    }

    pub(crate) fn dn_scope(&mut self) {
        if self.scope == 0 {
            panic!("Tried to exit scope of global environment");
        }

        // Drop variables created in this scope
        self.variables.remove(&self.scope);
        self.scope -= 1;
    }

    pub(crate) fn declare(
        &mut self,
        identifier: String,
        variable: Variable,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        if self
            .variables
            .get(&self.scope)
            .expect("variable store at scope does not exist")
            .contains_key(&identifier)
        {
            return Err(runtime_error!(
                range,
                RuntimeErrorKind::VariableAlreadyDefined,
                identifier
            ));
        }

        Store::check_type_compatible(&identifier, &variable, None)?;

        self.insert(identifier, variable);
        Ok(())
    }

    pub(crate) fn insert(&mut self, identifier: String, variable: Variable) {
        self.variables.entry(self.scope).and_modify(|m| {
            m.insert(identifier, variable);
        });
    }

    pub(crate) fn write(
        &mut self,
        identifier: String,
        value: (StoreValue, (usize, usize)),
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        let (variable, scope) = match self.search_scopes(&identifier, self.scope) {
            Some(v) => Ok(v),
            None => Err(runtime_error!(
                range,
                RuntimeErrorKind::UndefinedVariable,
                identifier
            )),
        }?;

        self.check_const(&identifier, variable, range)?;
        Store::check_type_compatible(&identifier, variable, Some(&value))?;

        self.variables.entry(scope).and_modify(|m| {
            m.entry(identifier)
                .and_modify(|variable| variable.value = Some((value.0.clone(), value.1)));
        });

        Ok(())
    }
    pub(crate) fn write_input(
        &mut self,
        identifier: String,
        input: String,
        range: (usize, usize),
    ) -> Result<(), RuntimeError> {
        let (variable, scope) = match self.search_scopes(&identifier, self.scope) {
            Some(v) => Ok(v),
            None => Err(runtime_error!(
                range,
                RuntimeErrorKind::UndefinedVariable,
                identifier
            )),
        }?;

        self.check_const(&identifier, variable, range)?;

        let store_value = self.try_marshalling_type(&identifier, input)?;
        let mut modified: Result<(), RuntimeError> = Ok(());

        self.variables.entry(scope).and_modify(|m| {
            m.entry(identifier.clone()).and_modify(|variable| {
                if variable.value.is_some()
                    && Store::check_type_compatible(
                        &identifier,
                        variable,
                        Some(&(store_value.clone(), range)),
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

                variable.value = Some((store_value, range));
            });
        });

        modified
    }

    pub(crate) fn get(&self, identifier: &str) -> Option<&Variable> {
        self.search_scopes(identifier, self.scope).map(|s| s.0)
    }

    pub(crate) fn check_type_compatible(
        identifier: &str,
        variable: &Variable,
        input: Option<&(StoreValue, (usize, usize))>,
    ) -> Result<(), RuntimeError> {
        let value_to_type_check = input.or(variable.value.as_ref());

        match value_to_type_check {
            // We can't type check an empty value,
            // expect later in the program to handle empty value cases
            None => Ok(()),
            Some(value) => match value.0 {
                StoreValue::Literal(ref literal) => {
                    Store::type_check_literal(identifier, &variable.type_, &literal, value.1)
                }
                StoreValue::Array(ref elements) => match &variable.type_ {
                    PolderType::Array(_) => elements.into_iter().try_for_each(|element| {
                        Store::type_check_literal(identifier, &variable.type_, &element, value.1)
                    }),
                    _ => Err(runtime_error!(
                        value.1,
                        RuntimeErrorKind::MismatchedType,
                        format!(
                            "Waarde is een {}, maar het type van {} is {}",
                            Into::<PolderType>::into(&value.0),
                            identifier,
                            variable.type_
                        )
                    )),
                },
            },
        }
    }

    fn search_scopes(&self, identifier: &str, scope: usize) -> Option<(&Variable, usize)> {
        match self
            .variables
            .get(&scope)
            .expect("variable store at scope does not exist")
            .get(identifier)
        {
            Some(var) => Some((var, scope)),
            None if scope == 0 => None,
            None => self.search_scopes(identifier, scope - 1),
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
            match literal_type {
                PolderType::Array(inner_type) => match inner_type {
                    Some(inner_type) => {
                        Store::type_check_literal(identifier, inner_type.as_ref(), value, range)
                        .map_err(|_| runtime_error!(
                                range,
                                RuntimeErrorKind::MismatchedType,
                                format!("{}:", value),
                                format!("{} is een {}, {} (van type {}) kan hier niet aan worden toegevoegd.", 
                                    identifier,
                                    literal_type,
                                    value,
                                    value.get_type(),
                                )
                            ))
                    }
                    None => Ok(()),
                },
                _ => Err(runtime_error!(
                    range,
                    RuntimeErrorKind::MismatchedType,
                    "",
                    format!(
                        "{} is een {}, geen {}",
                        identifier,
                        literal_type,
                        value.get_type()
                    )
                )),
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
                    variable.get_value().as_ref().unwrap().0
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
    pub(crate) fn get_value(&self) -> &Option<(StoreValue, (usize, usize))> {
        &self.value
    }
}
