use crate::{
    error::{self, Error},
    lexer::TokenType,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Variable(String),
    Array(Vec<Expr>),
    Nil,

    // Expressions
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    IndexAssign {
        array: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
    },
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::LessThan => write!(f, "<"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::GreaterThan => write!(f, ">"),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::LogicalAnd => write!(f, "and"),
            BinaryOp::LogicalOr => write!(f, "or"),
        }
    }
}

impl TryFrom<TokenType> for BinaryOp {
    type Error = error::Error;

    fn try_from(token: TokenType) -> Result<Self, Self::Error> {
        match token {
            TokenType::Plus => Ok(BinaryOp::Add),
            TokenType::Minus => Ok(BinaryOp::Subtract),
            TokenType::Star => Ok(BinaryOp::Multiply),
            TokenType::Slash => Ok(BinaryOp::Divide),
            TokenType::EqualEqual => Ok(BinaryOp::Equal),
            TokenType::BangEqual => Ok(BinaryOp::NotEqual),
            TokenType::LessThan => Ok(BinaryOp::LessThan),
            TokenType::LessEqual => Ok(BinaryOp::LessEqual),
            TokenType::GreaterThan => Ok(BinaryOp::GreaterThan),
            TokenType::GreaterEqual => Ok(BinaryOp::GreaterEqual),
            TokenType::And => Ok(BinaryOp::LogicalAnd),
            TokenType::Or => Ok(BinaryOp::LogicalOr),
            _ => Err(Error::internal(format!(
                "invalid token type for binary operator: {token:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Negate => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

impl TryFrom<TokenType> for UnaryOp {
    type Error = error::Error;

    fn try_from(token: TokenType) -> Result<Self, Self::Error> {
        match token {
            TokenType::Minus => Ok(UnaryOp::Negate),
            TokenType::Bang => Ok(UnaryOp::Not),
            _ => Err(Error::internal(format!(
                "invalid token type for unary operator: {token:?}"
            ))),
        }
    }
}

pub trait Visitor<T> {
    fn visit_number(&mut self, value: f64) -> T;
    fn visit_string(&mut self, value: &str) -> T;
    fn visit_boolean(&mut self, value: bool) -> T;
    fn visit_nil(&mut self) -> T;
    fn visit_identifier(&mut self, name: &str) -> T;
    fn visit_array(&mut self, elements: &[Expr]) -> T;
    fn visit_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> T;
    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> T;
    fn visit_assign(&mut self, name: &str, value: &Expr) -> T;
    fn visit_index_assign(&mut self, array: &Expr, index: &Expr, value: &Expr) -> T;
    fn visit_index(&mut self, array: &Expr, index: &Expr) -> T;
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Number(n) => visitor.visit_number(*n),
            Expr::String(s) => visitor.visit_string(s),
            Expr::Boolean(b) => visitor.visit_boolean(*b),
            Expr::Nil => visitor.visit_nil(),
            Expr::Variable(name) => visitor.visit_identifier(name),
            Expr::Array(elements) => visitor.visit_array(elements),
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary(left, operator, right),
            Expr::Unary { operator, operand } => visitor.visit_unary(operator, operand),
            Expr::Assign { name, value } => visitor.visit_assign(name, value),
            Expr::IndexAssign {
                array,
                index,
                value,
            } => visitor.visit_index_assign(array, index, value),
            Expr::Index { array, index } => visitor.visit_index(array, index),
            Expr::Call { callee, arguments } => visitor.visit_call(callee, arguments),
        }
    }
}
