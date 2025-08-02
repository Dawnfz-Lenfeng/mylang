use super::value::Value;
use crate::error::Error;

pub type InterpreterResult<T> = Result<T, RuntimeControl>;

#[derive(Debug, Clone)]
pub enum RuntimeControl {
    Error(Error),
    Return(Value),
    Break,
    Continue,
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
            RuntimeControl::Return(_) => {
                Error::runtime("'return' outside function".to_string())
            }
            RuntimeControl::Break => {
                Error::runtime("'break' outside loop".to_string())
            }
            RuntimeControl::Continue => {
                Error::runtime("'continue' outside loop".to_string())
            }
        }
    }
}
