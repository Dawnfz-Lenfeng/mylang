use crate::utils::{Position, Span};
use std::fmt;

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub message: String,
    pub error_type: ErrorType,
    pub span: Option<Span>,
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
            span: None,
            file: None,
        }
    }

    pub fn with_type(mut self, error_type: ErrorType) -> Self {
        self.error_type = error_type;
        self
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        let location = Position {
            line,
            column,
            offset: 0,
        };
        self.span = Some(Span {
            start: location,
            end: location,
        });
        self
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn lexical_error(message: String, span: Span) -> Self {
        Self {
            message,
            error_type: ErrorType::LexicalError,
            span: Some(span),
            file: None,
        }
    }

    pub fn lexical_error_with_span(message: String, span: Span) -> Self {
        Self {
            message,
            error_type: ErrorType::LexicalError,
            span: Some(span),
            file: None,
        }
    }

    pub fn syntax_error(message: String, span: Span) -> Self {
        Self {
            message,
            error_type: ErrorType::SyntaxError,
            span: Some(span),
            file: None,
        }
    }

    pub fn semantic_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::SemanticError,
            span: None,
            file: None,
        }
    }

    pub fn semantic_error_with_span(message: String, span: Span) -> Self {
        Self {
            message,
            error_type: ErrorType::SemanticError,
            span: Some(span),
            file: None,
        }
    }

    pub fn type_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::TypeError,
            span: None,
            file: None,
        }
    }

    pub fn name_error(message: String, span: Span) -> Self {
        Self {
            message,
            error_type: ErrorType::NameError,
            span: Some(span),
            file: None,
        }
    }

    pub fn io_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::IOError,
            span: None,
            file: None,
        }
    }

    pub fn runtime_error(message: String) -> Self {
        Self {
            message,
            error_type: ErrorType::InternalError, // or add a RuntimeError variant
            span: None,
            file: None,
        }
    }

    // Convenience methods for backward compatibility
    pub fn line(&self) -> Option<usize> {
        self.span.map(|s| s.start.line)
    }

    pub fn column(&self) -> Option<usize> {
        self.span.map(|s| s.start.column)
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

        if let Some(span) = self.span {
            if let Some(file) = &self.file {
                // Show range if start and end are different
                if span.start.line != span.end.line || span.start.column != span.end.column {
                    write!(
                        f,
                        "{}:{}:{}-{}:{}: {}: {}",
                        file,
                        span.start.line,
                        span.start.column,
                        span.end.line,
                        span.end.column,
                        error_type_str,
                        self.message
                    )
                } else {
                    write!(
                        f,
                        "{}:{}:{}: {}: {}",
                        file, span.start.line, span.start.column, error_type_str, self.message
                    )
                }
            } else {
                // Show range if start and end are different
                if span.start.line != span.end.line || span.start.column != span.end.column {
                    write!(
                        f,
                        "{}:{}-{}:{}: {}: {}",
                        span.start.line,
                        span.start.column,
                        span.end.line,
                        span.end.column,
                        error_type_str,
                        self.message
                    )
                } else {
                    write!(
                        f,
                        "{}:{}: {}: {}",
                        span.start.line, span.start.column, error_type_str, self.message
                    )
                }
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
