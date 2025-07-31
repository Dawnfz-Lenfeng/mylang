use super::{
    env::{EnvRef, Environment},
    value::Value,
};
use crate::{
    error::{Error, Result},
    parser::{
        expr::{self, Expr},
        stmt::{self, Stmt, Visitor},
        BinaryOp, UnaryOp,
    },
};

pub struct Interpreter {
    env: EnvRef,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new_global(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.env = Environment::new_local(&self.env);
    }

    pub fn exit_scope(&mut self) {
        let enclosing = self.env.borrow_mut().enclosing.take();
        self.env = enclosing.unwrap();
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        Ok(())
    }

    fn visit_print(&mut self, expr: &Expr) -> Result<()> {
        let value = expr.accept(self)?;
        println!("{value}");
        Ok(())
    }

    fn visit_var_decl(&mut self, name: &str, initializer: Option<&Expr>) -> Result<()> {
        let value = if let Some(expr) = initializer {
            expr.accept(self)?
        } else {
            Value::Nil
        };

        self.env.borrow_mut().define(name.to_string(), value);
        Ok(())
    }

    fn visit_func_decl(&mut self, name: &str, parameters: &[String], body: &Stmt) -> Result<()> {
        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<()> {
        Ok(())
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        Ok(())
    }

    fn visit_return(&mut self, value: Option<&Expr>) -> Result<()> {
        Ok(())
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Result<()> {
        Ok(())
    }
}

impl expr::Visitor<Result<Value>> for Interpreter {
    fn visit_string(&mut self, value: &str) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_number(&mut self, value: f64) -> Result<Value> {
        Ok(Value::Number(value))
    }

    fn visit_boolean(&mut self, value: bool) -> Result<Value> {
        Ok(Value::Boolean(value))
    }

    fn visit_identifier(&mut self, name: &str) -> Result<Value> {
        self.env.borrow().get(name)
    }

    fn visit_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<Value> {
        let left = left.accept(self)?;
        let right = right.accept(self)?;
        match op {
            BinaryOp::Add => Ok(Value::Nil),
            BinaryOp::Subtract => Ok(Value::Nil),
            BinaryOp::Multiply => Ok(Value::Nil),
            BinaryOp::Divide => Ok(Value::Nil),
            BinaryOp::Equal => Ok(Value::Nil),
            BinaryOp::NotEqual => Ok(Value::Nil),
            BinaryOp::LessThan => Ok(Value::Nil),
            BinaryOp::LessEqual => Ok(Value::Nil),
            BinaryOp::GreaterThan => Ok(Value::Nil),
            BinaryOp::GreaterEqual => Ok(Value::Nil),
            BinaryOp::LogicalAnd => Ok(Value::Nil),
            BinaryOp::LogicalOr => Ok(Value::Nil),
        }
    }

    fn visit_assign(&mut self, name: &str, value: &Expr) -> Result<Value> {
        let value = value.accept(self)?;
        self.env.borrow_mut().set(name, value.clone())?;
        Ok(value)
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> Result<Value> {
        Ok(Value::Nil)
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Result<Value> {
        Ok(Value::Nil)
    }

    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Value> {
        Ok(Value::Nil)
    }
}
