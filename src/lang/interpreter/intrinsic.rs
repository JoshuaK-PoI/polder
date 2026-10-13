use std::ops::Range;

use crate::{
    io::Io,
    lang::{
        compiler::{
            expression::ExpressionType, intrinsic::Intrinsics, statement::IntrinsicStatement,
        },
        interpreter::{runtime_error::RuntimeError, runtime_error::RuntimeErrorKind, store::Store},
    },
    runtime_error,
};

pub(crate) fn interpret(
    intrinsic: &IntrinsicStatement,
    range: &Range<usize>,
    store: &mut Store,
    io: &mut Io,
) -> Result<(), RuntimeError> {
    match intrinsic.intrinsic {
        Intrinsics::Print => print(intrinsic, range, true, store, io),
        Intrinsics::Write => print(intrinsic, range, false, store, io),
        Intrinsics::Readline => readline(intrinsic, range, store, io),
    }
}

fn print(
    intrinsic: &IntrinsicStatement,
    range: &Range<usize>,
    add_newline: bool,
    store: &mut Store,
    io: &mut Io,
) -> Result<(), RuntimeError> {
    if intrinsic.args.is_empty() {
        Err(runtime_error!(
            range.clone(),
            RuntimeErrorKind::NoArgumentsProvided,
            intrinsic.intrinsic
        ))
    } else {
        for arg in intrinsic.args.iter() {
            match arg.expression {
                ExpressionType::StringExpr(ref content) => {
                    io.write(content.text.as_str())
                        .map_err(|e| runtime_error!(range.clone(), RuntimeErrorKind::IOError))?;
                }
                ExpressionType::Identifier(ref content) => {
                    let value = store.get_variable_by_name(&content.name);

                    if value.is_none() {
                        return Err(runtime_error!(
                            range.clone(),
                            RuntimeErrorKind::UndefinedVariable,
                            content.name
                        ));
                    }

                    if value.unwrap().get_value().is_none() {
                        return Err(runtime_error!(
                            range.clone(),
                            RuntimeErrorKind::VariableHasNoValue,
                            content.name
                        ));
                    }

                    io.write(value.unwrap().get_value().as_ref().unwrap().as_str())
                        .map_err(|_| runtime_error!(range.clone(), RuntimeErrorKind::IOError))?;
                }
            }
        }

        if add_newline {
            io.write("\n")
                .map_err(|_| runtime_error!(range.clone(), RuntimeErrorKind::IOError))?;
        }
        Ok(())
    }
}

fn readline(
    intrinsic: &IntrinsicStatement,
    range: &Range<usize>,
    store: &mut Store,
    io: &mut Io,
) -> Result<(), RuntimeError> {
    let identifier = match intrinsic.args.get(0) {
        Some(ref expr) => match &expr.expression {
            ExpressionType::Identifier(ident) => Ok(ident.name.clone()),
            other => Err(runtime_error!(
                range.clone(),
                RuntimeErrorKind::UndefinedError
            )),
        },
        None => Err(runtime_error!(
            range.clone(),
            RuntimeErrorKind::NoArgumentsProvided
        )),
    }?;

    let input = io
        .read_line()
        .map_err(|e| runtime_error!(range.clone(), RuntimeErrorKind::IOError))?;
    store
        .write(&identifier, &input, range)
        .map_err(|e| runtime_error!(range.clone(), RuntimeErrorKind::UndefinedError, identifier))?;
    Ok(())
}
