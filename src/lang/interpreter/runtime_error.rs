#[macro_export]
macro_rules! runtime_error {
    ($kind:ident, $message:expr) => {
        RuntimeError {
            kind: RuntimeErrorKind::$kind,
            message: format!("{}", $message).trim().to_string(),
        }
    };

    ($kind:ident) => {
        RuntimeError {
            kind: RuntimeErrorKind::$kind,
            message: String::from(RuntimeErrorKind::$kind),
        }
    };
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeError {
    #[allow(dead_code)]
    pub(crate) kind: RuntimeErrorKind,
    pub(crate) message: String,
}

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        runtime_error!(IOError, value)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeErrorKind {
    UndefinedVariable,
    VariableHasNoValue,
    MismatchedType,
    NoArgumentsProvided,
    InvalidArgument,
    InvalidComparison,
    NoInputGiven,
    DivideByZero,
    CannotCalculateNonNumeric,
    UnsupportedFunction,
    IOError,
}

impl From<RuntimeErrorKind> for String {
    fn from(value: RuntimeErrorKind) -> Self {
        match value {
            RuntimeErrorKind::DivideByZero => format!("Kan niet door 0 delen."),
            _ => format!("{:?}", value),
        }
    }
}

pub(super) fn report(runtime_error: &RuntimeError) -> () {
    eprintln!("{}", runtime_error.message);
}

pub(super) type RuntimeResult<T = ()> = Result<T, RuntimeError>;
