use std::{collections::HashMap, ops::Range};

use crate::{
    lang::{
        compiler::parser::{
            statement::{VariableAssignmentStatement, VariableDeclarationStatement},
            types::PolderType,
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
    variable_type: PolderType,
    variable_ident: String,
    variable_value: Option<String>,
    is_const: bool,
}

impl From<&Variable> for String {
    fn from(value: &Variable) -> Self {
        format!(
            "<{}{}> {} {:<16.16}",
            if value.is_const { "(const) " } else { "" },
            value.variable_type,
            value.variable_ident,
            if value.variable_value.is_some() {
                format!("\"{}\"", value.variable_value.as_ref().unwrap())
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
        if self.variable_defined(&variable.variable_ident) {
            return Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::VariableAlreadyDefined,
                variable.variable_ident
            ));
        }

        // self.check_type_compatible(variable);

        self.variables
            .insert(variable.variable_ident.clone(), variable);
        Ok(())
    }

    pub(crate) fn update(
        &mut self,
        assignment: &VariableAssignmentStatement,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        self.write(&assignment.identifier, &assignment.value_literal, range)
    }

    pub(crate) fn write(
        &mut self,
        identifier: &String,
        value: &String,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        if !self.variable_defined(identifier) {
            return Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::UndefinedVariable,
                identifier
            ));
        }

        self.variables
            .entry(identifier.clone())
            .and_modify(|variable| variable.variable_value = Some(value.clone()));

        Ok(())
    }

    fn variable_defined(&self, identifier: &String) -> bool {
        self.get(identifier).is_some()
    }

    pub(crate) fn get(&self, identifier: &String) -> Option<&Variable> {
        self.variables.get(identifier)
    }
}

impl From<&VariableDeclarationStatement> for Variable {
    fn from(declaration: &VariableDeclarationStatement) -> Variable {
        Variable {
            variable_type: declaration.variable_type.clone(),
            variable_ident: declaration.identifier.clone(),
            variable_value: declaration.value_literal.clone(),
            is_const: declaration.is_const,
        }
    }
}

impl Variable {
    pub(crate) fn get_value(&self) -> &Option<String> {
        &self.variable_value
    }
}
