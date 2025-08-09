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
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    // Statements
    Expression(Expr),
    Print(Vec<Expr>),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Break,
    Continue,
    Return {
        value: Option<Expr>,
    },
}

pub trait Visitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_print(&mut self, exprs: &[Expr]) -> T;
    fn visit_var_decl(&mut self, name: &str, initializer: Option<&Expr>) -> T;
    fn visit_func_decl(&mut self, name: &str, params: &[String], body: &[Stmt]) -> T;
    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>)
        -> T;
    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> T;
    fn visit_return(&mut self, value: Option<&Expr>) -> T;
    fn visit_break(&mut self) -> T;
    fn visit_continue(&mut self) -> T;
    fn visit_block(&mut self, statements: &[Stmt]) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Expression(expr) => visitor.visit_expr(expr),
            Stmt::Print(exprs) => visitor.visit_print(exprs),
            Stmt::Block(statements) => visitor.visit_block(statements),
            Stmt::VarDecl { name, initializer } => {
                visitor.visit_var_decl(name, initializer.as_ref())
            }
            Stmt::FuncDecl { name, params, body } => visitor.visit_func_decl(name, params, body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if(condition, then_branch, else_branch.as_deref()),
            Stmt::While { condition, body } => visitor.visit_while(condition, body),
            Stmt::Return { value } => visitor.visit_return(value.as_ref()),
            Stmt::Break => visitor.visit_break(),
            Stmt::Continue => visitor.visit_continue(),
        }
    }
}
