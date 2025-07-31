#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),

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
    Grouping(Box<Expr>),
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

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Not,
}

pub trait Visitor<T> {
    fn visit_number(&mut self, value: f64) -> T;
    fn visit_string(&mut self, value: &str) -> T;
    fn visit_boolean(&mut self, value: bool) -> T;
    fn visit_identifier(&mut self, name: &str) -> T;
    fn visit_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> T;
    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_assign(&mut self, name: &str, value: &Expr) -> T;
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Number(n) => visitor.visit_number(*n),
            Expr::String(s) => visitor.visit_string(s),
            Expr::Boolean(b) => visitor.visit_boolean(*b),
            Expr::Identifier(name) => visitor.visit_identifier(name),
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary(left, operator, right),
            Expr::Unary { operator, operand } => visitor.visit_unary(operator, operand),
            Expr::Grouping(expr) => visitor.visit_grouping(expr),
            Expr::Assign { name, value } => visitor.visit_assign(name, value),
            Expr::Call { callee, arguments } => visitor.visit_call(callee, arguments),
        }
    }
}
