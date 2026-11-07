use crate::{
    lang::{
        compiler::{Span, parser::expression::literal::Literal},
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind, RuntimeResult},
            store::Variable,
        },
    },
    runtime_error,
};

pub(super) fn push(variable: &Variable, arguments: Vec<(Literal, Span)>) -> RuntimeResult<Literal> {
    match &variable.value {
        Some(Literal::String(str)) => {
            let arg = arguments
                .iter()
                .map(|a| a.0.clone())
                .map(|s| match s {
                    Literal::Vec(arr) => Err(runtime_error!(
                        MismatchedType,
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
                        MismatchedType,
                        format!("Kan geen {} toevoegen aan een tekst", lit)
                    )),
                    Err(err) => Err(err),
                });

            let arg = arg.collect::<Vec<RuntimeResult<String>>>();

            if arg.iter().any(|r| r.is_err()) {
                return Err(arg.iter().find_map(|r| r.as_ref().err()).unwrap().clone());
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
) -> RuntimeResult<Literal> {
    match &variable.value {
        Some(Literal::String(str)) => {
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

            match str.chars().nth(index as usize) {
                Some(value) => Ok(Literal::Char(value.to_owned())),
                None => Err(runtime_error!(
                    VariableHasNoValue,
                    format!(
                        "{} heeft geen waarde op plek {}",
                        variable_identifier, index
                    )
                )),
            }
        }
        None => Err(runtime_error!(VariableHasNoValue, variable_identifier)),
        _ => unreachable!(),
    }
}
