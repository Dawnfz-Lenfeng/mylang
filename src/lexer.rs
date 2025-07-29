use crate::error::CompilerError;
use crate::utils::{Position, Span};

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
    And,
    Or,
    Not,

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Assign,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Pipe,
    Ampersand,
    Arrow,

    // Delimiters
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
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

pub struct Lexer {
    input: Vec<char>,
    position: Position,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: Position::new(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        let mut tokens = Vec::new();
        while let Some((ch, position)) = self.consume_char() {
            if ch.is_whitespace() {
                continue;
            }
            if ch == '/' && self.peek() == Some('/') {
                self.skip_comment();
                continue;
            }
            tokens.push(self.read_token(ch, position)?);
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            span: Span {
                start: self.position,
                end: self.position,
            },
        });

        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position.offset).copied()
    }

    fn consume_char(&mut self) -> Option<(char, Position)> {
        let ch = self.peek()?;
        let position = self.position;
        self.advance();
        Some((ch, position))
    }

    fn advance(&mut self) {
        if let Some(ch) = self.peek() {
            self.position.advance(ch);
        }
    }

    fn skip_comment(&mut self) {
        while let Some((ch, ..)) = self.consume_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn read_token(&mut self, ch: char, start: Position) -> Result<Token, CompilerError> {
        let token_type = match ch {
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
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(TokenType::Arrow)
                } else {
                    Ok(TokenType::Minus)
                }
            }
            '*' => Ok(TokenType::Asterisk),
            '/' => Ok(TokenType::Slash),
            '%' => Ok(TokenType::Percent),
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(TokenType::Equal)
                } else {
                    Ok(TokenType::Assign)
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(TokenType::NotEqual)
                } else {
                    Ok(TokenType::Not)
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(TokenType::LessEqual)
                } else {
                    Ok(TokenType::LessThan)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(TokenType::GreaterEqual)
                } else {
                    Ok(TokenType::GreaterThan)
                }
            }
            '&' => Ok(TokenType::Pipe),
            '|' => Ok(TokenType::Ampersand),
            '"' | '\'' => self.read_string(ch),
            '0'..='9' => self.read_number(ch),
            'a'..='z' | 'A'..='Z' | '_' => Ok(self.read_identifier(ch)),
            _ => Err(CompilerError::lexical_error(
                format!("Unexpected character: {}", ch),
                Span {
                    start,
                    end: self.position,
                },
            )),
        };

        Ok(Token {
            token_type: token_type?,
            span: Span {
                start,
                end: self.position,
            },
        })
    }

    fn read_number(&mut self, ch: char) -> Result<TokenType, CompilerError> {
        let mut number = ch.to_string();
        let mut has_dot = false;
        while let Some(ch) = self.peek() {
            if ch == '.' {
                if has_dot {
                    return Err(CompilerError::lexical_error(
                        format!("Invalid number"),
                        Span {
                            start: self.position,
                            end: self.position,
                        },
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

        while let Some((ch, position)) = self.consume_char() {
            if ch == delimiter {
                return Ok(TokenType::String(string));
            }
            if ch == '\\' {
                if let Some((escaped_ch, ..)) = self.consume_char() {
                    string.push(escaped_ch);
                    continue;
                } else {
                    return Err(CompilerError::lexical_error(
                        "Unterminated string literal".to_string(),
                        Span {
                            start: position,
                            end: self.position,
                        },
                    ));
                }
            }
            if ch == '\n' {
                return Err(CompilerError::lexical_error(
                    "Unterminated string literal".to_string(),
                    Span {
                        start: position,
                        end: self.position,
                    },
                ));
            }
            string.push(ch);
        }

        Err(CompilerError::lexical_error(
            "Unterminated string literal".to_string(),
            Span {
                start: self.position,
                end: self.position,
            },
        ))
    }

    fn read_identifier(&mut self, ch: char) -> TokenType {
        let mut identifier = ch.to_string();
        while let Some(ch) = self.peek() {
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
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => TokenType::Identifier(identifier),
        }
    }
}
