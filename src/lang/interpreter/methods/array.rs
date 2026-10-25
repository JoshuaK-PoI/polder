use crate::{
    lang::{
        compiler::parser::expression::literal::Literal,
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::{Store, StoreValue, Variable},
        },
    },
    runtime_error,
};

pub(crate) fn length(
    variable: &Variable,
    variable_identifier: &String,
    span: (usize, usize),
) -> Result<StoreValue, RuntimeError> {
    match variable.get_value() {
        Some((StoreValue::Array(arr), _span)) => Ok(StoreValue::Literal(Literal::Integer(
            arr.len().try_into().unwrap(),
        ))),
        Some(store_value) => {
            todo!("Implement internal function 'lengte' for {}", store_value.0)
        }
        None => Err(runtime_error!(
            span,
            RuntimeErrorKind::VariableHasNoValue,
            variable_identifier
        )),
    }
}

pub(super) fn push(
    variable: &Variable,
    variable_identifier: &String,
    arguments: Vec<(StoreValue, (usize, usize))>,
    span: (usize, usize),
) -> Result<StoreValue, RuntimeError> {
    match variable.get_value() {
        Some((StoreValue::Array(arr), _)) => Ok({
            let mut arr = arr.clone();
            for value in arguments {
                Store::check_type_compatible(variable_identifier, variable, Some(&value))?;
                match value {
                    (StoreValue::Array(argument_arr), _span) => {
                        let mut argument_arr = argument_arr.clone();
                        arr.append(&mut argument_arr)
                    }
                    (StoreValue::Literal(lit), _span) => arr.push(lit),
                }
            }
            StoreValue::Array(arr.to_vec())
        }),
        None => {
            let mut vec: Vec<Literal> = Vec::new();

            for value in arguments {
                match value {
                    (StoreValue::Array(argument_arr), _span) => {
                        let mut argument_arr = argument_arr.clone();
                        vec.append(&mut argument_arr)
                    }
                    (StoreValue::Literal(lit), _span) => vec.push(lit),
                }
            }

            Ok(StoreValue::Array(vec))
        }
        Some((StoreValue::Literal(lit), _span)) => Err(runtime_error!(
            span,
            RuntimeErrorKind::UnsupportedFunction,
            lit,
            format!("Kan {} niet toevoegen aan een {}", lit, variable.type_)
        )),
    }
}
