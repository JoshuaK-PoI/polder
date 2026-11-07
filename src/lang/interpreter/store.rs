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

#[derive(Debug, Clone)]
pub(crate) struct Variable {
    pub(crate) type_: PolderType,
    pub(crate) value: Option<Literal>,
}

#[hotpath::measure_all]
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

        self.variables.remove(&self.scope);
        self.scope -= 1;
    }

    pub(crate) fn clean_scope(&mut self) {
        // Drop variables created in this scope
        self.variables.entry(self.scope).and_modify(|map| {
            map.clear();
        });
    }

    pub(crate) fn declare(&mut self, identifier: &String, variable: Variable) -> () {
        self.insert(identifier, variable);
    }

    pub(crate) fn insert(&mut self, identifier: &String, variable: Variable) {
        self.variables.entry(self.scope).and_modify(|m| {
            m.insert(identifier.to_string(), variable);
        });
    }

    pub(crate) fn write(
        &mut self,
        identifier: &String,
        value: Literal,
    ) -> Result<(), RuntimeError> {
        let (_variable, scope) = match self.search_scopes(&identifier, &self.scope) {
            Some(v) => Ok(v),
            None => Err(runtime_error!(UndefinedVariable, identifier)),
        }?;

        self.variables.entry(scope).and_modify(|m| {
            m.entry(identifier.clone())
                .and_modify(|variable| variable.value = Some(value.clone()));
        });

        Ok(())
    }
    pub(crate) fn write_input(
        &mut self,
        identifier: String,
        input: String,
    ) -> Result<(), RuntimeError> {
        let (_variable, scope) = match self.search_scopes(&identifier, &self.scope) {
            Some(v) => Ok(v),
            None => Err(runtime_error!(UndefinedVariable, identifier)),
        }?;

        let store_value = self.try_marshalling_type(&identifier, input)?;
        self.variables.entry(scope).and_modify(|m| {
            m.entry(identifier.clone()).and_modify(|variable| {
                variable.value = Some(store_value);
            });
        });

        Ok(())
    }

    pub(crate) fn get(&self, identifier: &str) -> &Variable {
        if let Some(var) = self.search_scopes(identifier, &self.scope).map(|s| s.0) {
            var
        } else {
            panic!("Attempted to access undefined variable {}", identifier)
        }
    }

    fn search_scopes(&self, identifier: &str, scope: &usize) -> Option<(&Variable, usize)> {
        match self
            .variables
            .get(scope)
            .expect("variable store at scope {scope} does not exist")
            .get(identifier)
        {
            Some(var) => Some((var, *scope)),
            None if *scope == 0 => None,
            None => self.search_scopes(identifier, &(scope - 1)),
        }
    }

    fn try_marshalling_type(
        &self,
        identifier: &str,
        input: String,
    ) -> Result<Literal, RuntimeError> {
        let type_ = &self.get(identifier).type_;

        match type_ {
            PolderType::Boolean => try_bool(input),
            PolderType::String => Ok(Literal::String(input)),
            PolderType::Integer => try_integer(input),
            PolderType::Float => try_float(input),
            PolderType::Char => try_char(input),
            PolderType::Array(_) => unimplemented!("Marshalling into array type"),
        }
    }
}

fn try_bool(input: String) -> Result<Literal, RuntimeError> {
    match input.as_str() {
        "waar" => Ok(Literal::Boolean(true)),
        "onwaar" => Ok(Literal::Boolean(false)),
        _ => Err(runtime_error!(
            MismatchedType,
            String::from("Verwachtte 'waar' of 'onwaar' hier")
        )),
    }
}

// @TODO: Deduplicate this code in future, has overlap with lexer
// @TODO: Runtime errors below should not abort the program but read from stdin again
fn try_integer(input: String) -> Result<Literal, RuntimeError> {
    match input.parse::<i64>() {
        Ok(int) => Ok(Literal::Integer(int)),
        Err(_) => Err(runtime_error!(
            MismatchedType,
            String::from("Verwachtte een getal hier")
        )),
    }
}

fn try_float(input: String) -> Result<Literal, RuntimeError> {
    match input.replace(',', ".").parse::<f64>() {
        Ok(flt) => Ok(Literal::Float(flt)),
        Err(_) => Err(runtime_error!(
            MismatchedType,
            String::from("Ongeldig karakter in het kommagetal")
        )),
    }
}

fn try_char(input: String) -> Result<Literal, RuntimeError> {
    match input.parse::<char>() {
        Ok(c) => Ok(Literal::Char(c)),
        Err(_) => Err(runtime_error!(
            MismatchedType,
            String::from("Verwachtte een enkel karakter hier")
        )),
    }
}

impl Variable {
    pub(crate) fn get_value(&self) -> Option<&Literal> {
        self.value.as_ref()
    }
}
