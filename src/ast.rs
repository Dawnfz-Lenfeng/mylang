#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Any,
    Number,
    String,
    Boolean,
    Array(Box<DataType>),
    Function(Vec<DataType>, Box<DataType>), // (parameters, return_type)
    Void,
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
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
    And,
    Or,
    Assign,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    // Variable declaration
    VarDecl {
        name: String,
        type_annotation: Option<DataType>,
        initializer: Option<Expr>,
    },

    // Function declaration
    FuncDecl {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<DataType>,
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

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<DataType>,
}

#[derive(Debug, Clone)]
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
