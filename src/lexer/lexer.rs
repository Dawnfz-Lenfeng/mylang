use super::token::{Token, TokenType};
use crate::{
    error::{Error, Result},
    utils::Location,
};

pub struct Lexer {
    input: Vec<char>,
    location: Location,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            location: Location::new(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        while let Some((ch, location)) = self.consume_char() {
            if ch.is_whitespace() {
                continue;
            }
            if ch == '/' && self.peek() == Some('/') {
                self.skip_comment();
                continue;
            }
            tokens.push(self.scan_token(ch, location)?);
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            location: self.location,
        });

        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.location.offset).copied()
    }

    fn consume_char(&mut self) -> Option<(char, Location)> {
        let ch = self.peek()?;
        let location = self.location.clone();
        self.advance();
        Some((ch, location))
    }

    fn advance(&mut self) {
        if let Some(ch) = self.peek() {
            self.location.advance(ch);
        }
    }

    fn skip_comment(&mut self) {
        while let Some((ch, ..)) = self.consume_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn scan_token(&mut self, ch: char, start: Location) -> Result<Token> {
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
            '-' => Ok(TokenType::Minus),
            '*' => Ok(TokenType::Star),
            '/' => Ok(TokenType::Slash),
            '=' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(TokenType::EqualEqual)
                }
                _ => Ok(TokenType::Equal),
            },
            '!' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(TokenType::BangEqual)
                }
                _ => Ok(TokenType::Bang),
            },
            '<' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(TokenType::LessEqual)
                }
                _ => Ok(TokenType::LessThan),
            },
            '>' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(TokenType::GreaterEqual)
                }
                _ => Ok(TokenType::GreaterThan),
            },
            '"' | '\'' => self.scan_string(start, ch),
            '0'..='9' => self.scan_number(start),
            'a'..='z' | 'A'..='Z' | '_' => Ok(self.scan_identifier(start)),
            _ => Err(Error::lexical(format!("Unexpected character: {ch}"), start)),
        }?;

        Ok(Token {
            token_type,
            location: start,
        })
    }

    fn scan_number(&mut self, start: Location) -> Result<TokenType> {
        while let Some(ch) = self.peek() {
            if !ch.is_ascii_digit() {
                break;
            }
            self.advance();
        }
        if let Some(ch) = self.peek() {
            if ch == '.' {
                self.advance();
            }
        }
        while let Some(ch) = self.peek() {
            if !ch.is_ascii_digit() {
                break;
            }
            self.advance();
        }
        let number = self.input[start.offset..self.location.offset]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .unwrap();
        Ok(TokenType::Number(number))
    }

    fn scan_string(&mut self, start: Location, delimiter: char) -> Result<TokenType> {
        while let Some((ch, ..)) = self.consume_char() {
            if ch == delimiter {
                let string = self.input[start.offset + 1..self.location.offset - 1]
                    .iter()
                    .collect::<String>();
                return Ok(TokenType::String(string));
            }
        }

        Err(Error::lexical(
            "Unterminated string literal".to_string(),
            self.location,
        ))
    }

    fn scan_identifier(&mut self, start: Location) -> TokenType {
        while let Some(ch) = self.peek() {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            self.advance();
        }
        let identifier = self.input[start.offset..self.location.offset]
            .iter()
            .collect::<String>();

        match identifier.as_str() {
            "let" => TokenType::Let,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            _ => TokenType::Identifier(identifier),
        }
    }
}
