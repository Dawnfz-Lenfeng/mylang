// Rust Compiler/Interpreter Library
// This crate provides the core functionality for lexical analysis, parsing,
// semantic analysis, and code generation.

pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod utils;

// Re-export commonly used types for convenience
pub use ast::{DataType, Expr, Program, Stmt};
pub use codegen::{CodeGenerator, TargetPlatform};
pub use error::{CompilerError, ErrorReporter, ErrorType};
pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;
pub use semantic::{SemanticAnalyzer, Symbol, SymbolTable};

/// The main compiler pipeline
pub struct Compiler {
    error_reporter: ErrorReporter,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            error_reporter: ErrorReporter::new(),
        }
    }

    /// Compile source code through the full pipeline
    pub fn compile(
        &mut self,
        source: &str,
        target: TargetPlatform,
    ) -> Result<String, CompilerError> {
        // Phase 1: Lexical Analysis
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.tokenize()?;

        // Phase 2: Syntax Analysis
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        // Phase 3: Semantic Analysis
        let mut semantic_analyzer = SemanticAnalyzer::new();
        semantic_analyzer.analyze(&ast)?;

        // Phase 4: Code Generation
        let mut code_generator = CodeGenerator::new(target);
        let output = code_generator.generate(&ast)?;

        Ok(output)
    }

    /// Get error reporter for additional error handling
    pub fn error_reporter(&self) -> &ErrorReporter {
        &self.error_reporter
    }

    /// Get mutable error reporter
    pub fn error_reporter_mut(&mut self) -> &mut ErrorReporter {
        &mut self.error_reporter
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
