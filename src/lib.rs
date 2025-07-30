// Rust Compiler/Interpreter Library
// This crate provides the core functionality for lexical analysis, parsing,
// semantic analysis, and code generation.

pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod utils;

// Re-export commonly used types for convenience
pub use ast::{DataType, Expr, Program, Stmt};
pub use error::{CompilerError, ErrorReporter, ErrorType};
pub use interpreter::interpreter::Interpreter;
pub use interpreter::utils::{Environment, Value};
pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;
pub use semantic::{SemanticAnalyzer, Symbol, SymbolTable};
