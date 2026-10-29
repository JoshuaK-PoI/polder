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

pub(super) fn push(
    variable: &Variable,
    variable_identifier: &String,
    arguments: Vec<(Literal, Span)>,
    span: Span,
) -> Result<Literal, RuntimeError> {
    match &variable.value {
        Some((Literal::String(str), _)) => {
            // unpack args

            let arg = arguments
                .iter()
                .map(|a| a.0.clone())
                .map(|s| match s {
                    Literal::Vec(arr) => Err(runtime_error!(
                        span,
                        RuntimeErrorKind::MismatchedType,
                        variable_identifier,
                        format!(
                            "Kan geen lijst van {} toevoegen aan een tekst",
                            if let Some(v) = arr.get(0) {
                                v.get_type()
                            } else {
                                "<onbekend>".to_string()
                            }
                        )
                    )),
                    lit => Ok(lit),
                })
                .map(|lit| match lit {
                    Ok(Literal::Char(c)) => Ok(c.to_string()),
                    Ok(Literal::String(s)) => Ok(s),
                    Ok(lit) => Err(runtime_error!(
                        span,
                        RuntimeErrorKind::MismatchedType,
                        variable_identifier,
                        format!("Kan geen {} toevoegen aan een tekst", lit)
                    )),
                    Err(err) => Err(err),
                });

            let arg = arg.collect::<Vec<Result<String, RuntimeError>>>();

            if arg.iter().any(|r| r.is_err()) {
                return Err(arg.get(0).unwrap().as_ref().err().unwrap().clone());
            }

            let arg = arg
                .into_iter()
                .map(|c| c.ok().unwrap())
                .collect::<Vec<String>>()
                .join("");

            let str = format!("{}{}", str, arg);

            Ok(Literal::String(str))
        }
        None => {
            let arg = arguments
                .iter()
                .map(|a| a.0.clone())
                .map(|lit| match lit {
                    Literal::Char(c) => c.to_string(),
                    Literal::String(s) => s,
                    _ => todo!(),
                })
                .collect::<String>();

            Ok(Literal::String(arg))
        }
        _ => unreachable!(),
    }
}

pub(super) fn get(
    variable: &Variable,
    variable_identifier: &String,
    arguments: Vec<(Literal, Span)>,
    span: Span,
) -> Result<Literal, RuntimeError> {
    match &variable.value {
        Some((Literal::String(str), _)) => {
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

            match str.chars().nth(index as usize) {
                Some(value) => Ok(Literal::Char(value.to_owned())),
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
        None => Err(runtime_error!(
            span,
            RuntimeErrorKind::VariableHasNoValue,
            variable_identifier
        )),
        _ => unreachable!(),
    }
}
