use crate::{
    io::Io,
    lang::{
        compiler::parser::expression::literal::Literal,
        interpreter::runtime_error::{
            RuntimeError,
            RuntimeErrorKind,
            RuntimeResult,
        },
    },
    runtime_error,
};

pub(super) fn print(literal: Vec<Literal>, add_newline: bool, io: &mut Io) -> RuntimeResult {
    for l in literal {
        io.write(&l.to_string())?;
    }

    if add_newline {
        io.write("\n")?;
    }

    Ok(())
}

pub(super) fn readline(io: &mut Io) -> RuntimeResult<String> {
    match io.read_line()? {
        Some(input) => Ok(input),
        None => Err(runtime_error!(NoInputGiven)),
    }
}
