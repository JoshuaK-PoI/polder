use std::ops::Range;

use crate::{
    lang::{
        compiler::{
            statement::{VariableAssignmentStatement, VariableDeclarationStatement},
            types::PolderType,
        },
        interpreter::runtime_error::{RuntimeError, RuntimeErrorKind},
    },
    runtime_error,
};

pub(crate) struct Store {
    variables: Vec<Variable>,
}

#[derive(Debug, Clone)]
pub(crate) struct Variable {
    variable_type: Option<PolderType>,
    variable_ident: String,
    variable_value: Option<String>,
    is_const: bool,
}

impl From<&Variable> for String {
    fn from(value: &Variable) -> Self {
        format!(
            "<{}{}> {} {:<16.16}",
            if value.is_const { "(const) " } else { "" },
            if value.variable_type.is_some() {
                value.variable_type.as_ref().unwrap()
            } else {
                &PolderType::Uninitialized
            },
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
            variables: Vec::new(),
        }
    }

    pub(crate) fn declare(
        &mut self,
        variable: Variable,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        self.check_variable_not_already_defined(&variable, range)?;
        self.variables.push(variable);

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
        match self
            .variables
            .iter_mut()
            .find(|v| v.variable_ident == *identifier)
        {
            Some(v) => {
                v.variable_value = Some(value.clone());
                Ok(())
            }
            None => Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::UndefinedVariable,
                identifier
            )),
        }
    }

    fn check_variable_not_already_defined(
        &self,
        variable: &Variable,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        if self
            .variables
            .iter()
            .filter(|v| v.variable_ident == variable.variable_ident)
            .collect::<Vec<&Variable>>()
            .len()
            > 0
        {
            Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::VariableAlreadyDefined,
                variable.variable_ident
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_variable_by_name(&self, name: &String) -> Option<&Variable> {
        self.variables.iter().find(|v| name.eq(&v.variable_ident))
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
