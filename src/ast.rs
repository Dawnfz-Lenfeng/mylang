use std::path::Display;

#[derive(Debug, Clone)]
pub enum DataType {
    Any,
    Number,
    String,
    Boolean,
    Array(Box<DataType>),
    Function(Vec<DataType>, Box<DataType>), // (parameters, return_type)
    Void,
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DataType::Any, _) | (_, DataType::Any) => true,
            (DataType::Array(a), DataType::Array(b)) => a == b,
            (DataType::Function(params_a, ret_a), DataType::Function(params_b, ret_b)) => {
                params_a == params_b && ret_a == ret_b
            }
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // Variable declaration
    VarDecl {
        name: String,
        type_annotation: Option<Expr>,
        initializer: Option<Expr>,
        is_mutable: bool,
    },

    // Function declaration
    FuncDecl {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Expr>,
        body: Vec<Stmt>,
    },

    // Control flow
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },

    While {
        condition: Expr,
        body: Vec<Stmt>,
    },

    For {
        name: String,
        collection: Expr,
        body: Vec<Stmt>,
    },

    Return {
        value: Option<Expr>,
    },

    // Expression statement
    Expression(Expr),

    // Block statement
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, stmt: Stmt) {
        self.statements.push(stmt);
    }
}
