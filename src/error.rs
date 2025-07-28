use std::fmt;

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub message: String,
    pub error_type: ErrorType,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    LexicalError,
    SyntaxError,
    SemanticError,
    TypeError,
    NameError,
    IOError,
    InternalError,
}

impl CompilerError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::InternalError,
            line: None,
            column: None,
            file: None,
        }
    }

    pub fn with_type(mut self, error_type: ErrorType) -> Self {
        self.error_type = error_type;
        self
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn lexical_error(message: String, line: usize, column: usize) -> Self {
        Self {
            message,
            error_type: ErrorType::LexicalError,
            line: Some(line),
            column: Some(column),
            file: None,
        }
    }

    pub fn syntax_error(message: String, line: usize, column: usize) -> Self {
        Self {
            message,
            error_type: ErrorType::SyntaxError,
            line: Some(line),
            column: Some(column),
            file: None,
        }
    }

    pub fn semantic_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::SemanticError,
            line: None,
            column: None,
            file: None,
        }
    }

    pub fn type_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::TypeError,
            line: None,
            column: None,
            file: None,
        }
    }

    pub fn name_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::NameError,
            line: None,
            column: None,
            file: None,
        }
    }

    pub fn io_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::IOError,
            line: None,
            column: None,
            file: None,
        }
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_type_str = match self.error_type {
            ErrorType::LexicalError => "Lexical Error",
            ErrorType::SyntaxError => "Syntax Error",
            ErrorType::SemanticError => "Semantic Error",
            ErrorType::TypeError => "Type Error",
            ErrorType::NameError => "Name Error",
            ErrorType::IOError => "I/O Error",
            ErrorType::InternalError => "Internal Error",
        };

        if let (Some(line), Some(column)) = (self.line, self.column) {
            if let Some(file) = &self.file {
                write!(
                    f,
                    "{}:{}:{}: {}: {}",
                    file, line, column, error_type_str, self.message
                )
            } else {
                write!(
                    f,
                    "{}:{}: {}: {}",
                    line, column, error_type_str, self.message
                )
            }
        } else {
            write!(f, "{}: {}", error_type_str, self.message)
        }
    }
}

impl std::error::Error for CompilerError {}

pub type Result<T> = std::result::Result<T, CompilerError>;

// Error reporting utilities
pub struct ErrorReporter {
    errors: Vec<CompilerError>,
    warnings: Vec<CompilerError>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: CompilerError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: CompilerError) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn get_errors(&self) -> &[CompilerError] {
        &self.errors
    }

    pub fn get_warnings(&self) -> &[CompilerError] {
        &self.warnings
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }

    pub fn report_all(&self) {
        for warning in &self.warnings {
            eprintln!("Warning: {}", warning);
        }

        for error in &self.errors {
            eprintln!("Error: {}", error);
        }

        if self.has_errors() {
            eprintln!("\nCompilation failed with {} error(s)", self.error_count());
            if self.has_warnings() {
                eprintln!("and {} warning(s)", self.warning_count());
            }
        } else if self.has_warnings() {
            eprintln!(
                "\nCompilation completed with {} warning(s)",
                self.warning_count()
            );
        }
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}
