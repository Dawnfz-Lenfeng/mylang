use std::fs;
use std::io;
use std::path::Path;

/// Read file contents as a string
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Write string contents to file
pub fn write_string_to_file<P: AsRef<Path>>(path: P, contents: &str) -> io::Result<()> {
    fs::write(path, contents)
}

/// Check if a file exists
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists() && path.as_ref().is_file()
}

/// Get file extension from path
pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Position tracking for lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    pub fn advance(&mut self, ch: char) {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    pub fn advance_by(&mut self, text: &str) {
        for ch in text.chars() {
            self.advance(ch);
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// String utilities
pub mod string_utils {
    /// Escape string for output
    pub fn escape_string(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\\' => "\\\\".to_string(),
                '"' => "\\\"".to_string(),
                '\'' => "\\'".to_string(),
                c if c.is_control() => format!("\\x{:02x}", c as u8),
                c => c.to_string(),
            })
            .collect()
    }

    /// Unescape string from input
    pub fn unescape_string(s: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some('x') => {
                        // Handle \xNN hex escape
                        let hex1 = chars.next().ok_or("Invalid hex escape")?;
                        let hex2 = chars.next().ok_or("Invalid hex escape")?;
                        let hex_str = format!("{}{}", hex1, hex2);
                        let byte =
                            u8::from_str_radix(&hex_str, 16).map_err(|_| "Invalid hex escape")?;
                        result.push(byte as char);
                    }
                    Some(c) => return Err(format!("Invalid escape sequence: \\{}", c)),
                    None => return Err("Unterminated escape sequence".to_string()),
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Check if character is valid identifier start
    pub fn is_identifier_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    /// Check if character is valid identifier continuation
    pub fn is_identifier_continue(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Check if string is a valid identifier
    pub fn is_valid_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();
        if let Some(first) = chars.next() {
            if !is_identifier_start(first) {
                return false;
            }
        }

        chars.all(is_identifier_continue)
    }
}

/// Number parsing utilities
pub mod number_utils {
    use std::str::FromStr;

    /// Parse integer from string with different bases
    pub fn parse_integer(s: &str) -> Result<i64, String> {
        if s.starts_with("0x") || s.starts_with("0X") {
            // Hexadecimal
            i64::from_str_radix(&s[2..], 16).map_err(|_| "Invalid hexadecimal number".to_string())
        } else if s.starts_with("0b") || s.starts_with("0B") {
            // Binary
            i64::from_str_radix(&s[2..], 2).map_err(|_| "Invalid binary number".to_string())
        } else if s.starts_with('0') && s.len() > 1 && s.chars().all(|c| c.is_ascii_digit()) {
            // Octal
            i64::from_str_radix(s, 8).map_err(|_| "Invalid octal number".to_string())
        } else {
            // Decimal
            s.parse::<i64>()
                .map_err(|_| "Invalid decimal number".to_string())
        }
    }

    /// Parse floating-point number from string
    pub fn parse_float(s: &str) -> Result<f64, String> {
        f64::from_str(s).map_err(|_| "Invalid floating-point number".to_string())
    }

    /// Check if string represents a valid number
    pub fn is_valid_number(s: &str) -> bool {
        parse_integer(s).is_ok() || parse_float(s).is_ok()
    }
}

/// Debugging utilities
pub mod debug_utils {
    use crate::ast::{Expr, Program, Stmt};

    /// Pretty-print AST for debugging
    pub fn pretty_print_program(program: &Program) -> String {
        let mut output = String::new();
        output.push_str("Program {\n");
        for (i, stmt) in program.statements.iter().enumerate() {
            output.push_str(&format!(
                "  Statement {}: {}\n",
                i,
                pretty_print_stmt(stmt, 2)
            ));
        }
        output.push('}');
        output
    }

    fn pretty_print_stmt(stmt: &Stmt, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                initializer,
                is_mutable,
            } => {
                let mut result = format!("VarDecl {{ name: {}", name);
                if let Some(typ) = type_annotation {
                    result.push_str(&format!(", type: {:?}", typ));
                }
                if let Some(init) = initializer {
                    result.push_str(&format!(", init: {}", pretty_print_expr(init, indent + 1)));
                }
                result.push_str(&format!(", is_mutable: {}", is_mutable));
                result.push_str(" }");
                result
            }
            Stmt::FuncDecl {
                name,
                parameters,
                return_type,
                body,
            } => {
                let mut result = format!("FuncDecl {{ name: {}, params: [", name);
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!("{}: {:?}", param.name, param.param_type));
                }
                result.push(']');
                if let Some(ret_type) = return_type {
                    result.push_str(&format!(", return_type: {:?}", ret_type));
                }
                result.push_str(", body: [\n");
                for stmt in body {
                    result.push_str(&format!(
                        "{}{},\n",
                        indent_str,
                        pretty_print_stmt(stmt, indent + 1)
                    ));
                }
                result.push_str(&format!("{}]", "  ".repeat(indent - 1)));
                result.push_str(" }");
                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut result = format!(
                    "If {{ condition: {}, then: [",
                    pretty_print_expr(condition, indent + 1)
                );
                for stmt in then_branch {
                    result.push_str(&format!(
                        "\n{}{},",
                        indent_str,
                        pretty_print_stmt(stmt, indent + 1)
                    ));
                }
                result.push(']');
                if let Some(else_stmts) = else_branch {
                    result.push_str(", else: [");
                    for stmt in else_stmts {
                        result.push_str(&format!(
                            "\n{}{},",
                            indent_str,
                            pretty_print_stmt(stmt, indent + 1)
                        ));
                    }
                    result.push(']');
                }
                result.push_str(" }");
                result
            }
            Stmt::While { condition, body } => {
                let mut result = format!(
                    "While {{ condition: {}, body: [",
                    pretty_print_expr(condition, indent + 1)
                );
                for stmt in body {
                    result.push_str(&format!(
                        "\n{}{},",
                        indent_str,
                        pretty_print_stmt(stmt, indent + 1)
                    ));
                }
                result.push_str("] }");
                result
            }
            Stmt::Return { value } => {
                if let Some(val) = value {
                    format!("Return {{ value: {} }}", pretty_print_expr(val, indent))
                } else {
                    "Return { value: None }".to_string()
                }
            }
            Stmt::Expression(expr) => {
                format!("Expression({})", pretty_print_expr(expr, indent))
            }
            Stmt::Block(stmts) => {
                let mut result = "Block [".to_string();
                for stmt in stmts {
                    result.push_str(&format!(
                        "\n{}{},",
                        indent_str,
                        pretty_print_stmt(stmt, indent + 1)
                    ));
                }
                result.push_str("]");
                result
            }
            _ => "TODO: Implement statement pretty printing".to_string(),
        }
    }

    fn pretty_print_expr(expr: &Expr, _indent: usize) -> String {
        match expr {
            Expr::Number(n) => format!("Number({})", n),
            Expr::String(s) => format!("String(\"{}\")", s),
            Expr::Boolean(b) => format!("Boolean({})", b),
            Expr::Identifier(name) => format!("Identifier({})", name),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "Binary({} {:?} {})",
                    pretty_print_expr(left, 0),
                    operator,
                    pretty_print_expr(right, 0)
                )
            }
            Expr::Unary { operator, operand } => {
                format!("Unary({:?} {})", operator, pretty_print_expr(operand, 0))
            }
            Expr::Call { callee, arguments } => {
                let mut result = format!("Call({}, [", pretty_print_expr(callee, 0));
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&pretty_print_expr(arg, 0));
                }
                result.push_str("])");
                result
            }
            Expr::Assign { name, value } => {
                format!("Assign({} = {})", name, pretty_print_expr(value, 0))
            }
            _ => "TODO: Implement expression pretty printing".to_string(),
        }
    }
}

/// Performance utilities
pub mod perf_utils {
    use std::time::{Duration, Instant};

    /// Simple timer for measuring compilation phases
    pub struct Timer {
        start: Instant,
        phase: String,
    }

    impl Timer {
        pub fn start(phase: String) -> Self {
            Self {
                start: Instant::now(),
                phase,
            }
        }

        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }

        pub fn stop(self) -> Duration {
            let elapsed = self.elapsed();
            println!("{} took: {:?}", self.phase, elapsed);
            elapsed
        }
    }

    /// Memory usage tracking (basic)
    pub struct MemoryTracker {
        initial_memory: usize,
    }

    impl MemoryTracker {
        pub fn new() -> Self {
            Self {
                initial_memory: 0, // TODO: Implement actual memory tracking
            }
        }

        pub fn memory_used(&self) -> usize {
            // TODO: Implement actual memory tracking
            0
        }
    }

    impl Default for MemoryTracker {
        fn default() -> Self {
            Self::new()
        }
    }
}
