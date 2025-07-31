#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),

    // Binary operations
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },

    // Unary operations
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },

    // Function call
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },

    // Array access
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    // Assignment
    Assign {
        name: String,
        value: Box<Expr>,
    },

    Array {
        elements: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    BitAnd,
    BitOr,
    LogicalAnd,
    LogicalOr,
    Assign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Not,
}