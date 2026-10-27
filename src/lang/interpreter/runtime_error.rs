use crate::logger::PolderError;

#[macro_export]
macro_rules! runtime_error {
    ($span: expr, $kind:expr, $message:expr, $formatter:expr) => {
        RuntimeError {
            message: format!("{} {}", $message, String::from($kind)).trim().to_string(),
            span_message: $formatter,
            span: $span,
        }
    };

    ($span: expr, $kind:expr, $message:expr) => {
        RuntimeError {
            message: format!("{} {}", $message, String::from($kind)).trim().to_string(),
            span_message: format!("{} {}", $message, String::from($kind)).trim().to_string(),
            span: $span,
        }
    };

    ($span: expr, $kind:expr) => {
        RuntimeError {
            message: format!("{}", String::from($kind)).trim().to_string(),
            span_message: format!("{}", String::from($kind)).trim().to_string(),
            span: $span,
        }
    };
}

pub(crate) struct RuntimeError {
    pub(crate) message: String,
    pub(crate) span_message: String,
    pub(crate) span: (usize, usize),
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

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        runtime_error!((0, 0), RuntimeErrorKind::IOError, value)
    }
}
pub(crate) enum RuntimeErrorKind {
    UndefinedVariable,
    VariableAlreadyDefined,
    VariableHasNoValue,
    VariableIsConst,
    MismatchedType,
    NoArgumentsProvided,
    InvalidArgument,
    InvalidComparison,
    NoInputGiven,
    DivideByZero,
    CannotCalculateNonNumeric,
    CannotOperateOnArray,
    UnsupportedFunction,
    IOError,
}

impl From<RuntimeErrorKind> for String {
    fn from(kind: RuntimeErrorKind) -> String {
        match kind {
            RuntimeErrorKind::UndefinedVariable => "bestaat niet".to_string(),
            RuntimeErrorKind::VariableAlreadyDefined => "bestaat al".to_string(),
            RuntimeErrorKind::VariableHasNoValue => "heeft nog geen waarde".to_string(),
            RuntimeErrorKind::VariableIsConst => "kan niet opnieuw worden toegewezen".to_string(),
            RuntimeErrorKind::MismatchedType => "Verkeerd type".to_string(),
            RuntimeErrorKind::NoArgumentsProvided => "Geen argumenten gespecificeerd".to_string(),
            RuntimeErrorKind::InvalidArgument => "Ongeldig argument".to_string(),
            RuntimeErrorKind::NoInputGiven => "Geef een waarde op".to_string(),
            RuntimeErrorKind::InvalidComparison => "Kan geen vergelijking tussen verschillende types uitvoeren".to_string(),
            RuntimeErrorKind::DivideByZero => "Kan niet door nul delen".to_string(),
            RuntimeErrorKind::CannotCalculateNonNumeric => {
                "Rekenen kan alleen met getallen of kommagetallen".to_string()
            }
            RuntimeErrorKind::CannotOperateOnArray => "Deze operatie is niet geldig met lijsten".to_string(),
            RuntimeErrorKind::UnsupportedFunction => "Functie (nog) niet ondersteund".to_string(),
            RuntimeErrorKind::IOError => "I/O Error".to_string(),
        }
    }
}
