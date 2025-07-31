use crate::utils::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),
    Boolean(bool),

    // Keywords
    Let,
    Const,
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
