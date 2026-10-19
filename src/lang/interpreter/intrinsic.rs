use std::ops::Range;

use crate::{
    io::Io,
    lang::{
        compiler::parser::expression::literal::Literal,
        interpreter::runtime_error::{RuntimeError, RuntimeErrorKind},
    },
    runtime_error,
};

pub(super) fn print(
    literal: Vec<Literal>,
    add_newline: bool,
    io: &mut Io,
) -> Result<(), RuntimeError> {
    for l in literal {
        io.write(&l.to_string())?;
    }

    if add_newline {
        io.write("\n")?;
    }

    Ok(())
}

pub(super) fn readline(range: &Range<usize>, io: &mut Io) -> Result<String, RuntimeError> {
    match io.read_line()? {
        Some(input) => Ok(input),
        None => Err(runtime_error!(
            range.clone(),
            RuntimeErrorKind::NoInputGiven
        )),
    }
}
