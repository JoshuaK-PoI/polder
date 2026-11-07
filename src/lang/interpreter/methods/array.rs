use crate::{
    lang::{
        compiler::{Span, parser::expression::literal::Literal},
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::Variable,
        },
    },
    runtime_error,
};

pub(crate) fn length(
    variable: &Variable,
    variable_identifier: &String,
) -> Result<Literal, RuntimeError> {
    match variable.get_value() {
        Some(Literal::Vec(arr)) => Ok(Literal::Integer(arr.len().try_into().unwrap())),
        Some(store_value) => {
            todo!("Implement internal function 'lengte' for {}", store_value)
        }
        None => Err(runtime_error!(
            VariableHasNoValue,
            format!("{} heeft geen waarde", variable_identifier)
        )),
    }
}

pub(super) fn push(
    variable: &Variable,
    arguments: Vec<(Literal, Span)>,
) -> Result<Literal, RuntimeError> {
    match variable.get_value() {
        Some(Literal::Vec(arr)) => Ok({
            let mut arr = arr.clone();
            for value in arguments {
                match value.0 {
                    Literal::Vec(argument_arr) => {
                        let mut argument_arr = argument_arr.clone();
                        arr.append(&mut argument_arr)
                    }
                    lit => arr.push(lit),
                }
            }
            Literal::Vec(arr.to_vec())
        }),
        None => {
            let mut vec: Vec<Literal> = Vec::new();

            for value in arguments {
                match value.0 {
                    Literal::Vec(argument_arr) => {
                        let mut argument_arr = argument_arr.clone();
                        vec.append(&mut argument_arr)
                    }
                    lit => vec.push(lit),
                }
            }

            Ok(Literal::Vec(vec))
        }
        Some(lit) => Err(runtime_error!(
            UnsupportedFunction,
            format!("Kan {} niet toevoegen aan een {}", lit, variable.type_)
        )),
    }
}

pub(super) fn get(
    variable: &Variable,
    variable_identifier: &String,
    arguments: Vec<(Literal, Span)>,
) -> Result<Literal, RuntimeError> {
    let arr = match variable.value {
        Some(ref value) => match value {
            Literal::Vec(arr) => arr,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    let index = match arguments.get(0) {
        Some(store_value) => match store_value.0 {
            Literal::Integer(int) => Ok(int),
            _ => Err(runtime_error!(
                InvalidArgument,
                format!("Kan alleen indexeren met een getal")
            )),
        },
        None => Err(runtime_error!(
            NoArgumentsProvided,
            format!(
                "Verwachtte een argument om {} mee te indexeren",
                variable_identifier
            )
        )),
    }?;

    if index < 0 {
        return Err(runtime_error!(
            InvalidArgument,
            format!(
                "{} kan niet geindexeerd worden met een negatief getal",
                variable_identifier
            )
        ));
    }

    match arr.get(index as usize) {
        Some(value) => Ok(value.to_owned()),
        None => Err(runtime_error!(
            VariableHasNoValue,
            format!(
                "{} heeft geen waarde op plek {}",
                variable_identifier, index
            )
        )),
    }
}
