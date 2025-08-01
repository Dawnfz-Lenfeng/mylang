use crate::{
    error::{self, Error},
    lexer::TokenType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Variable(String),

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
                "Invalid token type for binary operator: {token:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Not,
}

impl TryFrom<TokenType> for UnaryOp {
    type Error = error::Error;

    fn try_from(token: TokenType) -> Result<Self, Self::Error> {
        match token {
            TokenType::Minus => Ok(UnaryOp::Minus),
            TokenType::Bang => Ok(UnaryOp::Not),
            _ => Err(Error::internal(format!(
                "Invalid token type for unary operator: {token:?}"
            ))),
        }
    }
}

pub trait Visitor<T> {
    fn visit_number(&mut self, value: f64) -> T;
    fn visit_string(&mut self, value: &str) -> T;
    fn visit_boolean(&mut self, value: bool) -> T;
    fn visit_identifier(&mut self, name: &str) -> T;
    fn visit_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> T;
    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> T;
    fn visit_assign(&mut self, name: &str, value: &Expr) -> T;
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Number(n) => visitor.visit_number(*n),
            Expr::String(s) => visitor.visit_string(s),
            Expr::Boolean(b) => visitor.visit_boolean(*b),
            Expr::Variable(name) => visitor.visit_identifier(name),
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary(left, operator, right),
            Expr::Unary { operator, operand } => visitor.visit_unary(operator, operand),
            Expr::Assign { name, value } => visitor.visit_assign(name, value),
            Expr::Call { callee, arguments } => visitor.visit_call(callee, arguments),
        }
    }
}
