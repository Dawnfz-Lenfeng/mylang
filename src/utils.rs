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

