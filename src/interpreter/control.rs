use super::value::Value;
use crate::error::Error;

pub type InterpreterResult<T> = Result<T, RuntimeControl>;

#[derive(Debug, Clone)]
pub enum RuntimeControl {
    Error(Error),
    Return(Value),
}

impl RuntimeControl {
    pub fn is_return(&self) -> bool {
        matches!(self, RuntimeControl::Return(_))
    }
}

impl From<Error> for RuntimeControl {
    fn from(error: Error) -> Self {
        RuntimeControl::Error(error)
    }
}

impl From<RuntimeControl> for Error {
    fn from(control: RuntimeControl) -> Self {
        match control {
            RuntimeControl::Error(error) => error,
            RuntimeControl::Return(value) => Error::runtime(format!("Unexpected return: {}", value)),
        }
    }
}
