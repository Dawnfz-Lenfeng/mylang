use crate::utils::Location;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Lexical,
    Syntax,
    Runtime,
    Io,
    Internal,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
    pub error_type: ErrorType,
    pub location: Option<Location>,
    pub file: Option<String>,
}

impl Error {
    pub fn new(error_type: ErrorType, message: String) -> Self {
        Self {
            message,
            error_type,
            location: None,
            file: None,
        }
    }

    pub fn with_location(error_type: ErrorType, message: String, location: Location) -> Self {
        Self {
            message,
            error_type,
            location: Some(location),
            file: None,
        }
    }

    pub fn with_file_location(
        error_type: ErrorType,
        message: String,
        file: String,
        location: Location,
    ) -> Self {
        Self {
            message,
            error_type,
            location: Some(location),
            file: Some(file),
        }
    }

    pub fn in_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn at_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    pub fn lexical(message: String, location: Location) -> Self {
        Self::with_location(ErrorType::Lexical, message, location)
    }

    pub fn syntax(message: String, location: Location) -> Self {
        Self::with_location(ErrorType::Syntax, message, location)
    }

    pub fn runtime(message: String) -> Self {
        Self::new(ErrorType::Runtime, message)
    }

    pub fn runtime_at(message: String, location: Location) -> Self {
        Self::with_location(ErrorType::Runtime, message, location)
    }

    pub fn io(message: String) -> Self {
        Self::new(ErrorType::Io, message)
    }

    pub fn internal(message: String) -> Self {
        Self::new(ErrorType::Internal, message)
    }

    pub fn line(&self) -> Option<usize> {
        self.location.map(|loc| loc.line)
    }

    pub fn column(&self) -> Option<usize> {
        self.location.map(|loc| loc.column)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self.error_type {
            ErrorType::Lexical => "Lexical error",
            ErrorType::Syntax => "Syntax error",
            ErrorType::Runtime => "Runtime error",
            ErrorType::Io => "I/O error",
            ErrorType::Internal => "Internal error",
        };

        match (&self.file, &self.location) {
            (Some(file), Some(location)) => {
                write!(
                    f,
                    "{}:{}:{}: {}: {}",
                    file, location.line, location.column, type_name, self.message
                )
            }
            (None, Some(location)) => {
                write!(
                    f,
                    "{}:{}: {}: {}",
                    location.line, location.column, type_name, self.message
                )
            }
            (Some(file), None) => {
                write!(f, "{}: {}: {}", file, type_name, self.message)
            }
            (None, None) => {
                write!(f, "{}: {}", type_name, self.message)
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::io(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeControl {
    Return(crate::interpreter::value::Value),
    Break,
    Continue,
}

impl RuntimeControl {
    pub fn is_return(&self) -> bool {
        matches!(self, RuntimeControl::Return(_))
    }

    pub fn get_return_value(self) -> Option<crate::interpreter::value::Value> {
        match self {
            RuntimeControl::Return(value) => Some(value),
            _ => None,
        }
    }
}

impl fmt::Display for RuntimeControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeControl::Return(value) => write!(f, "return {}", value),
            RuntimeControl::Break => write!(f, "break"),
            RuntimeControl::Continue => write!(f, "continue"),
        }
    }
}
