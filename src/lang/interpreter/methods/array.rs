use crate::{
    lang::{
        compiler::{Span, parser::expression::literal::Literal},
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::{Store, Variable},
        },
    },
    runtime_error,
};

pub(crate) fn length(
    variable: &Variable,
    variable_identifier: &String,
    span: Span,
) -> Result<Literal, RuntimeError> {
    match variable.get_value() {
        Some((Literal::Vec(arr), _span)) => Ok(Literal::Integer(arr.len().try_into().unwrap())),
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
    arguments: Vec<(Literal, Span)>,
    span: Span,
) -> Result<Literal, RuntimeError> {
    match variable.get_value() {
        Some((Literal::Vec(arr), _)) => Ok({
            let mut arr = arr.clone();
            for value in arguments {
                Store::check_type_compatible(variable_identifier, variable, Some(&value))?;
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
        Some((lit, _)) => Err(runtime_error!(
            span,
            RuntimeErrorKind::UnsupportedFunction,
            lit,
            format!("Kan {} niet toevoegen aan een {}", lit, variable.type_)
        )),
    }
}

pub(super) fn get(
    variable: &Variable,
    variable_identifier: &String,
    arguments: Vec<(Literal, Span)>,
    span: Span,
) -> Result<Literal, RuntimeError> {
    let arr = match variable.value {
        Some(ref value) => match value.0 {
            Literal::Vec(ref arr) => arr,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    let index = match arguments.get(0) {
        Some(store_value) => match store_value.0 {
            Literal::Integer(int) => Ok(int),
            _ => Err(runtime_error!(
                span,
                RuntimeErrorKind::InvalidArgument,
                variable_identifier,
                format!("Kan alleen indexeren met een getal")
            )),
        },
        None => Err(runtime_error!(
            span,
            RuntimeErrorKind::NoArgumentsProvided,
            variable_identifier,
            format!(
                "Verwachtte een argument om {} mee te indexeren",
                variable_identifier
            )
        )),
    }?;

    if index < 0 {
        return Err(runtime_error!(
            span,
            RuntimeErrorKind::InvalidArgument,
            variable_identifier,
            format!(
                "{} kan niet geindexeerd worden met een negatief getal",
                variable_identifier
            )
        ));
    }

    match arr.get(index as usize) {
        Some(value) => Ok(value.to_owned()),
        None => Err(runtime_error!(
            span,
            RuntimeErrorKind::VariableHasNoValue,
            variable_identifier,
            format!(
                "{} heeft geen waarde op plek {}",
                variable_identifier, index
            )
        )),
    }
}
