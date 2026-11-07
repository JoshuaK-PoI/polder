mod array;
mod string;

use crate::{
    lang::{
        compiler::{
            Span,
            parser::{expression::literal::Literal, types::PolderType},
        },
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::Variable,
        },
    },
    runtime_error,
};

pub(crate) fn call_method(
    variable: &Variable,
    variable_identifier: &String,
    method: &String,
    arguments: Vec<(Literal, Span)>,
) -> Result<Literal, RuntimeError> {
    match &variable.type_ {
        PolderType::Array(_) => match method.as_str() {
            "lengte" => array::length(variable, variable_identifier),
            "Voeg toe" => array::push(variable, arguments),
            "element" => array::get(variable, variable_identifier, arguments),
            fun => Err(runtime_error!(
                UnsupportedFunction,
                format!(
                    "Methode '{}' is geen functie voor een {}",
                    fun, variable.type_
                )
            )),
        },
        PolderType::String => match method.as_str() {
            "Voeg toe" => string::push(variable, arguments),
            "element" => string::get(variable, variable_identifier, arguments),
            fun => Err(runtime_error!(
                UnsupportedFunction,
                format!(
                    "Methode '{}' is geen functie voor een {}",
                    fun, variable.type_
                )
            )),
        },
        val => todo!("Implement call_method for {:?}", val),
    }
}
