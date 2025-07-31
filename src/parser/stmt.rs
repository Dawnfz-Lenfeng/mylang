use super::expr::Expr;

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
