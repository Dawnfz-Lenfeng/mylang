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
    Compilation,    // 编译错误
    VmRuntime,      // 虚拟机运行时错误
    StackOverflow,  // 栈溢出
    StackUnderflow, // 栈下溢
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

    /// Create a compilation error
    pub fn compilation(message: String) -> Self {
        Self::new(ErrorType::Compilation, message)
    }

    /// Create a compilation error with location
    pub fn compilation_at(message: String, location: Location) -> Self {
        Self::with_location(ErrorType::Compilation, message, location)
    }

    /// Create a VM runtime error
    pub fn vm_runtime(message: String) -> Self {
        Self::new(ErrorType::VmRuntime, message)
    }

    /// Create a VM runtime error with location
    pub fn vm_runtime_at(message: String, location: Location) -> Self {
        Self::with_location(ErrorType::VmRuntime, message, location)
    }

    /// Create a stack overflow error
    pub fn stack_overflow(message: String) -> Self {
        Self::new(ErrorType::StackOverflow, message)
    }

    /// Create a stack underflow error
    pub fn stack_underflow() -> Self {
        Self::new(ErrorType::StackUnderflow, "Stack underflow".to_string())
    }

    /// Create a constant pool overflow error
    pub fn constant_overflow() -> Self {
        Self::compilation("Too many constants in chunk (max 256)".to_string())
    }

    /// Create an invalid opcode error
    pub fn invalid_opcode(opcode: u8) -> Self {
        Self::vm_runtime(format!("Invalid opcode: {}", opcode))
    }

    /// Create a type error for operations
    pub fn type_error(operation: &str, expected: &str, found: &str) -> Self {
        Self::vm_runtime(format!(
            "Type error in {}: expected {}, found {}",
            operation, expected, found
        ))
    }

    /// Create an arity error for function calls
    pub fn arity_error(function: &str, expected: usize, found: usize) -> Self {
        Self::vm_runtime(format!(
            "Arity error in function '{}': expected {} arguments, found {}",
            function, expected, found
        ))
    }

    /// Create a division by zero error
    pub fn division_by_zero() -> Self {
        Self::vm_runtime("Division by zero".to_string())
    }

    /// Create an array index out of bounds error
    pub fn index_out_of_bounds(index: usize, length: usize) -> Self {
        Self::vm_runtime(format!(
            "Array index {} out of bounds (length: {})",
            index, length
        ))
    }

    /// Create an undefined variable error
    pub fn undefined_variable(name: &str) -> Self {
        Self::vm_runtime(format!("Undefined variable '{name}'"))
    }

    /// Create an undefined function error
    pub fn undefined_function(name: &str) -> Self {
        Self::vm_runtime(format!("Undefined function '{name}'"))
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
            ErrorType::Lexical => "Lexical Error",
            ErrorType::Syntax => "Syntax Error",
            ErrorType::Runtime => "Runtime Error",
            ErrorType::Io => "IO Error",
            ErrorType::Internal => "Internal Error",
            ErrorType::Compilation => "Compilation Error",
            ErrorType::VmRuntime => "VMRuntime Error",
            ErrorType::StackOverflow => "StackOverflow Error",
            ErrorType::StackUnderflow => "StackUnderflow Error",
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

/// VM-specific error utilities
impl Error {
    /// Check if this is a VM runtime error
    pub fn is_vm_runtime(&self) -> bool {
        matches!(self.error_type, ErrorType::VmRuntime)
    }

    /// Check if this is a compilation error
    pub fn is_compilation(&self) -> bool {
        matches!(self.error_type, ErrorType::Compilation)
    }

    /// Check if this is a stack-related error
    pub fn is_stack_error(&self) -> bool {
        matches!(
            self.error_type,
            ErrorType::StackOverflow | ErrorType::StackUnderflow
        )
    }

    /// Create an error with bytecode instruction pointer context
    pub fn with_ip(mut self, ip: usize) -> Self {
        self.message = format!("{} (at instruction {})", self.message, ip);
        self
    }

    /// Create an error with stack size context
    pub fn with_stack_size(mut self, size: usize) -> Self {
        self.message = format!("{} (stack size: {})", self.message, size);
        self
    }

    /// Create an error with call frame context
    pub fn with_frame_info(mut self, frame_count: usize, function: &str) -> Self {
        self.message = format!(
            "{} (in function '{}', frame {})",
            self.message, function, frame_count
        );
        self
    }

    /// Add VM debugging information
    pub fn with_vm_debug(mut self, ip: usize, stack_size: usize, frame_count: usize) -> Self {
        self.message = format!(
            "{} [VM: ip={}, stack={}, frames={}]",
            self.message, ip, stack_size, frame_count
        );
        self
    }
}

/// Runtime control flow for VM (similar to interpreter control)
#[derive(Debug, Clone)]
pub enum VmControl {
    Continue,
    Break,
    Return(crate::treewalk_interpreter::Value),
    Error(Error),
}

impl From<Error> for VmControl {
    fn from(error: Error) -> Self {
        VmControl::Error(error)
    }
}

impl From<VmControl> for Result<()> {
    fn from(control: VmControl) -> Self {
        match control {
            VmControl::Continue => Ok(()),
            VmControl::Break => Err(Error::vm_runtime("Unexpected break".to_string())),
            VmControl::Return(_) => Err(Error::vm_runtime("Unexpected return".to_string())),
            VmControl::Error(e) => Err(e),
        }
    }
}
