use crate::utils::Location;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Nil,

    // Keywords
    Let,
    Fn,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    Return,
    And,
    Or,
    Print,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

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

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub location: Location,
}
