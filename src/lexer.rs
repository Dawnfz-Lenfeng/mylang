use crate::error::CompilerError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),
    Boolean(bool),

    // Keywords
    Let,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Return,

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
    Not,

    // Delimiters
    Arrow,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,

    // Special
    Unknown,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // 保存token开始位置
            let start_line = self.line;
            let start_column = self.column;

            self.advance();

            if ch == '/' && self.current_char() == Some('/') {
                self.skip_comment();
                continue;
            }

            let token_type = self.read_token_type(ch)?;
            if token_type == TokenType::Unknown {
                return Err(CompilerError::lexical_error(
                    format!("Unexpected character: {}", ch),
                    start_line,
                    start_column,
                ));
            }

            tokens.push(Token {
                token_type,
                line: start_line,
                column: start_column,
            });
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_token_type(&mut self, ch: char) -> Result<TokenType, CompilerError> {
        match ch {
            '(' => Ok(TokenType::LeftParen),
            ')' => Ok(TokenType::RightParen),
            '{' => Ok(TokenType::LeftBrace),
            '}' => Ok(TokenType::RightBrace),
            '[' => Ok(TokenType::LeftBracket),
            ']' => Ok(TokenType::RightBracket),
            ',' => Ok(TokenType::Comma),
            ';' => Ok(TokenType::Semicolon),
            ':' => Ok(TokenType::Colon),
            '+' => Ok(TokenType::Plus),
            '-' => {
                if self.current_char() == Some('>') {
                    self.advance();
                    Ok(TokenType::Arrow)
                } else {
                    Ok(TokenType::Minus)
                }
            }
            '*' => Ok(TokenType::Multiply),
            '/' => Ok(TokenType::Divide),
            '%' => Ok(TokenType::Modulo),
            '=' => {
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(TokenType::Equal)
                } else {
                    Ok(TokenType::Assign)
                }
            }
            '!' => {
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(TokenType::NotEqual)
                } else {
                    Ok(TokenType::Not)
                }
            }
            '<' => {
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(TokenType::LessEqual)
                } else {
                    Ok(TokenType::LessThan)
                }
            }
            '>' => {
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(TokenType::GreaterEqual)
                } else {
                    Ok(TokenType::GreaterThan)
                }
            }
            '&' => {
                if self.current_char() == Some('&') {
                    self.advance();
                    Ok(TokenType::And)
                } else {
                    Err(CompilerError::lexical_error(
                        "Expected '&&'".to_string(),
                        self.line,
                        self.column,
                    ))
                }
            }
            '|' => {
                if self.current_char() == Some('|') {
                    self.advance();
                    Ok(TokenType::Or)
                } else {
                    Err(CompilerError::lexical_error(
                        "Expected '||'".to_string(),
                        self.line,
                        self.column,
                    ))
                }
            }
            '"' | '\'' => self.read_string(ch),
            '0'..='9' => self.read_number(ch),
            'a'..='z' | 'A'..='Z' | '_' => Ok(self.read_identifier(ch)),
            _ => Ok(TokenType::Unknown),
        }
    }

    fn read_number(&mut self, ch: char) -> Result<TokenType, CompilerError> {
        let mut number = ch.to_string();
        let mut has_dot = false;
        while let Some(ch) = self.current_char() {
            if ch == '.' {
                if has_dot {
                    return Err(CompilerError::lexical_error(
                        format!("Invalid number"),
                        self.line,
                        self.column,
                    ));
                }
                has_dot = true;
                number.push(ch);
                self.advance();
                continue;
            }
            if !ch.is_numeric() {
                break;
            }
            number.push(ch);
            self.advance();
        }
        Ok(TokenType::Number(number.parse().unwrap()))
    }

    fn read_string(&mut self, delimiter: char) -> Result<TokenType, CompilerError> {
        let mut string = String::new();

        while let Some(ch) = self.current_char() {
            if ch == delimiter {
                self.advance();
                return Ok(TokenType::String(string));
            }
            if ch == '\\' {
                self.advance();
                if let Some(escaped_ch) = self.current_char() {
                    string.push(escaped_ch);
                    self.advance();
                    continue;
                } else {
                    return Err(CompilerError::lexical_error(
                        "Unterminated string literal".to_string(),
                        self.line,
                        self.column,
                    ));
                }
            }
            if ch == '\n' {
                return Err(CompilerError::lexical_error(
                    "Unterminated string literal".to_string(),
                    self.line,
                    self.column,
                ));
            }
            string.push(ch);
            self.advance();
        }

        Err(CompilerError::lexical_error(
            "Unterminated string literal".to_string(),
            self.line,
            self.column,
        ))
    }

    fn read_identifier(&mut self, ch: char) -> TokenType {
        let mut identifier = ch.to_string();
        while let Some(ch) = self.current_char() {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            identifier.push(ch);
            self.advance();
        }

        match identifier.as_str() {
            "let" => TokenType::Let,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "return" => TokenType::Return,
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            _ => TokenType::Identifier(identifier),
        }
    }
}
