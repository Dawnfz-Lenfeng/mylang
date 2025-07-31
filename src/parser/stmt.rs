use super::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // Declarations
    VarDecl {
        name: String,
        initializer: Option<Expr>,
    },
    FuncDecl {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },

    // Statements
    Expression(Expr),
    Print(Expr),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
    },
}
