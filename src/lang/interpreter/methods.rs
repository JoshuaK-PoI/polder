use crate::{
    lang::{
        compiler::parser::{expression::literal::Literal, types::PolderType}, interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind}, store::{Store, StoreValue, Variable},
        },
    }, runtime_error,
};

impl StoreValue {
    pub(crate) fn call_method(
        variable: &Variable,
        variable_identifier: &String,
        method: &String,
        arguments: Vec<(StoreValue, (usize, usize))>,
        span: (usize, usize),
    ) -> Result<StoreValue, RuntimeError> {
        match &variable.type_ {
            PolderType::Array(inner_type) => StoreValue::call_array_method(
                variable,
                variable_identifier,
                method,
                arguments,
                inner_type.as_ref().unwrap(),
                span,
            ),
            val => todo!("Implement call_method for {:?}", val),
        }
    }

    fn call_array_method(
        variable: &Variable,
        variable_identifier: &String,
        method: &String,
        arguments: Vec<(StoreValue, (usize, usize))>,
        array_type: &Box<PolderType>,
        span: (usize, usize),
    ) -> Result<StoreValue, RuntimeError> {
        match method.as_str() {
            "lengte" => match variable.get_value() {
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
            },
            "Voeg toe" => match variable.get_value() {
                Some((StoreValue::Array(arr), _span)) => Ok({
                    let mut arr = arr.clone();
                    for value in arguments {
                        Store::check_type_compatible(variable_identifier, variable, Some(&value), span)?;
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
                    method,
                    format!("Kan {} niet toevoegen aan een {}", lit, variable.type_)
                )),
            },
            m => Err(runtime_error!(
                span,
                RuntimeErrorKind::UnsupportedFunction,
                method,
                format!(
                    "'{}' is geen bekende methode voor een {}",
                    m, variable.type_
                )
            )),
        }
    }
}
