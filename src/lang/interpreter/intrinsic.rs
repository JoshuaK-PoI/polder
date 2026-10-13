use std::ops::Range;

use crate::{
    io::Io,
    lang::{
        compiler::{
            CompilerError, ErrorCode, expression::ExpressionType, intrinsic::Intrinsics,
            statement::IntrinsicStatement,
        },
        interpreter::store::Store,
    },
};

pub(crate) fn interpret(
    intrinsic: &IntrinsicStatement,
    range: &Range<usize>,
    store: &mut Store,
    io: &mut Io,
) -> Result<(), CompilerError> {
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
) -> Result<(), CompilerError> {
    if intrinsic.args.is_empty() {
        Err(CompilerError {
            error_code: ErrorCode::NoArgumentsProvided,
            error_message: format!("No arguments provided for print statement"),
            span_message: format!("Expected something to print, got nothing"),
            span: range.clone(),
            help: None,
            info: None,
        })
    } else {
        for arg in intrinsic.args.iter() {
            match arg.expression {
                ExpressionType::StringExpr(ref content) => {
                    io.write(content.text.as_str()).map_err(|e| CompilerError {
                        error_code: ErrorCode::InternalCompilerError,
                        error_message: e.clone(),
                        span_message: e,
                        span: range.clone(),
                        help: None,
                        info: None,
                    })?
                }
                ExpressionType::Identifier(ref content) => {
                    let value = store.get_variable_by_name(&content.name);
                    if value.is_none() {
                        return Err(CompilerError {
                            error_code: ErrorCode::UndefinedVariable,
                            error_message: format!("Call to undefined variable {} ", &content.name),
                            span_message: format!("{} is not defined", &content.name),
                            span: range.clone(),
                            help: None,
                            info: None,
                        });
                    }
                    io.write(value.unwrap().get_value().as_ref().unwrap().as_str())
                        .map_err(|e| CompilerError {
                            error_code: ErrorCode::InternalCompilerError,
                            error_message: e.clone(),
                            span_message: e,
                            span: range.clone(),
                            help: None,
                            info: None,
                        })?;
                }
            }
        }

        if add_newline {
            io.write("\n").map_err(|e| CompilerError {
                error_code: ErrorCode::InternalCompilerError,
                error_message: e.clone(),
                span_message: e,
                span: range.clone(),
                help: None,
                info: None,
            })?;
        }
        Ok(())
    }
}

fn readline(
    intrinsic: &IntrinsicStatement,
    range: &Range<usize>,
    store: &mut Store,
    io: &mut Io,
) -> Result<(), CompilerError> {
    let identifier = match intrinsic.args.get(0) {
        Some(ref expr) => match &expr.expression {
            ExpressionType::Identifier(ident) => Ok(ident.name.clone()),
            other => Err(CompilerError {
                error_code: ErrorCode::InvalidExpression,
                error_message: format!("Expected an identifier, got {:?}", other),
                span_message: format!("Expected variable to read into, got nothing"),
                span: range.clone(),
                help: None,
                info: None,
            }),
        },
        None => Err(CompilerError {
            error_code: ErrorCode::NoArgumentsProvided,
            error_message: format!("No arguments provided for readline statement"),
            span_message: format!("Expected variable to read into, got nothing"),
            span: range.clone(),
            help: None,
            info: None,
        }),
    }?;

    let input = io.read_line().map_err(|e| CompilerError {
        error_code: ErrorCode::InternalCompilerError,
        error_message: format!("Expected to read from input but errored: {}", e.clone()),
        span_message: e,
        span: range.clone(),
        help: None,
        info: None,
    })?;

    store.write(&identifier, &input, range)?;
    Ok(())
}
