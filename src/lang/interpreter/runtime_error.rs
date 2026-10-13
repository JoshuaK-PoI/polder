use std::ops::Range;

use crate::logger::PolderError;

#[macro_export]
macro_rules! runtime_error {
    ($span: expr, $kind:expr, $message:expr) => {
        RuntimeError {
            kind: $kind,
            message: format!("{} {}", $message, String::from($kind)),
            span_message: format!("{} {}", $message, String::from($kind)),
            span: $span,
        }
    };

    ($span: expr, $kind:expr) => {
        RuntimeError {
            kind: $kind,
            message: format!("{}", String::from($kind)),
            span_message: format!("{}", String::from($kind)),
            span: $span,
        }
    };
}

pub(crate) struct RuntimeError {
    pub(crate) kind: RuntimeErrorKind,
    pub(crate) message: String,
    pub(crate) span_message: String,
    pub(crate) span: Range<usize>,
}

impl From<RuntimeError> for PolderError {
    fn from(value: RuntimeError) -> Self {
        PolderError {
            message: value.message,
            span_message: value.span_message,
            span: value.span,
        }
    }
}
pub(crate) enum RuntimeErrorKind {
    UndefinedVariable,
    VariableAlreadyDefined,
    VariableHasNoValue,
    NoArgumentsProvided,
    IOError,
    UndefinedError,
}

impl From<RuntimeErrorKind> for String {
    fn from(kind: RuntimeErrorKind) -> String {
        match kind {
            RuntimeErrorKind::UndefinedVariable => "bestaat niet".to_string(),
            RuntimeErrorKind::VariableAlreadyDefined => "bestaat al".to_string(),
            RuntimeErrorKind::VariableHasNoValue => "heeft nog geen waarde".to_string(),
            RuntimeErrorKind::NoArgumentsProvided => "Geen argumenten gespecificeerd".to_string(),
            RuntimeErrorKind::IOError => "I/O Error".to_string(),
            RuntimeErrorKind::UndefinedError => "Undefined Error".to_string(),
        }
    }
}
