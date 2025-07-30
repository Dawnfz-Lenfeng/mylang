use crate::interpreter::utils::Value;
use std::fmt;

/// Runtime errors and control flow for the interpreter
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// Regular runtime error
    Error { message: String },
    /// Return statement - not really an error, but uses error propagation
    Return { value: Value },
    /// Break statement (for future loop control)
    Break,
    /// Continue statement (for future loop control)  
    Continue,
}

impl RuntimeError {
    pub fn error(message: String) -> Self {
        Self::Error { message }
    }

    pub fn return_value(value: Value) -> Self {
        Self::Return { value }
    }

    pub fn is_return(&self) -> bool {
        matches!(self, RuntimeError::Return { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, RuntimeError::Error { .. })
    }

    pub fn get_return_value(self) -> Option<Value> {
        match self {
            RuntimeError::Return { value } => Some(value),
            _ => None,
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Error { message } => write!(f, "Runtime Error: {}", message),
            RuntimeError::Return { value } => write!(f, "Return: {}", value),
            RuntimeError::Break => write!(f, "Break"),
            RuntimeError::Continue => write!(f, "Continue"),
        }
    }
}

impl std::error::Error for RuntimeError {}
