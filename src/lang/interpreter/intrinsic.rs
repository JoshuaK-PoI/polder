use std::ops::Range;

use crate::{
    io::Io,
    lang::{
        compiler::parser::{
            expression::ExpressionType, intrinsic::Intrinsics, statement::IntrinsicStatement,
        },
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::Store,
        },
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
    for arg in intrinsic.args.iter() {
        match arg.expression {
            ExpressionType::StringExpr(ref content) => io.write(content.text.as_str())?,
            ExpressionType::Identifier(ref content) => match store.get(&content.name) {
                Some(variable) => match variable.get_value() {
                    Some(value) => Ok(io.write(value)?),
                    None => Err(runtime_error!(
                        range.clone(),
                        RuntimeErrorKind::VariableHasNoValue,
                        content.name
                    )),
                },
                None => Err(runtime_error!(
                    range.clone(),
                    RuntimeErrorKind::UndefinedVariable,
                    content.name
                )),
            }?,
        };
    }

    if add_newline {
        io.write("\n")?;
    }
    Ok(())
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
                RuntimeErrorKind::MismatchedType,
                other
            )),
        },
        None => Err(runtime_error!(
            range.clone(),
            RuntimeErrorKind::NoArgumentsProvided
        )),
    }?;

    let input = io.read_line()?;
    if input.is_none() {
        Err(runtime_error!(
            range.clone(),
            RuntimeErrorKind::NoInputGiven
        ))
    } else {
        store.write(&identifier, &input.unwrap(), range)
    }
}
