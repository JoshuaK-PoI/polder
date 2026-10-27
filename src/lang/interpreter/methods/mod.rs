mod array;

use crate::{
    lang::{
        compiler::parser::types::PolderType,
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::{StoreValue, Variable},
        },
    },
    runtime_error,
};

pub(crate) fn call_method(
    variable: &Variable,
    variable_identifier: &String,
    method: &String,
    arguments: Vec<(StoreValue, (usize, usize))>,
    span: (usize, usize),
) -> Result<StoreValue, RuntimeError> {
    match &variable.type_ {
        PolderType::Array(_) => match method.as_str() {
            "lengte" => array::length(variable, variable_identifier, span),
            "Voeg toe" => array::push(variable, variable_identifier, arguments, span),
            "element" => array::get(variable, variable_identifier, arguments, span),
            fun => Err(runtime_error!(
                span,
                RuntimeErrorKind::UnsupportedFunction,
                fun,
                format!("Methode '{}' is geen functie voor een {}", fun, variable.type_)
            )),
        },
        val => todo!("Implement call_method for {:?}", val),
    }
}
