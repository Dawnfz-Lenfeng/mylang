use super::{
    control::{InterpreterResult, RuntimeControl},
    env::{EnvRef, Environment},
    value::Value,
};
use crate::{
    error::{Error, Result},
    parser::{
        expr::{self, Expr},
        stmt::{self, Stmt},
        BinaryOp, UnaryOp,
    },
};
use std::rc::Rc;

pub struct Interpreter {
    env: EnvRef,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new_global(),
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            stmt.accept(self)?;
        }
        Ok(())
    }

    pub fn enter_scope(&mut self) {
        self.env = Environment::new_local(&self.env);
    }

    pub fn exit_scope(&mut self) {
        let enclosing = self.env.borrow_mut().enclosing.take();
        self.env = enclosing.unwrap();
    }
}

impl stmt::Visitor<InterpreterResult<()>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> InterpreterResult<()> {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_print(&mut self, exprs: &[Expr]) -> InterpreterResult<()> {
        let output = exprs
            .iter()
            .map(|expr| expr.accept(self).map(|value| value.to_string()))
            .collect::<Result<Vec<_>>>()?
            .join(" ");

        println!("{output}");
        Ok(())
    }

    fn visit_var_decl(&mut self, name: &str, initializer: Option<&Expr>) -> InterpreterResult<()> {
        let value = if let Some(expr) = initializer {
            expr.accept(self)?
        } else {
            Value::Nil
        };

        self.env.borrow_mut().define(name.to_string(), value);
        Ok(())
    }

    fn visit_func_decl(
        &mut self,
        name: &str,
        params: &[String],
        body: &Stmt,
    ) -> InterpreterResult<()> {
        let func = Value::Function {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.clone(),
            closure: Rc::clone(&self.env),
        };
        self.env.borrow_mut().define(name.to_string(), func);
        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> InterpreterResult<()> {
        if condition.accept(self)?.is_truthy() {
            then_branch.accept(self)?;
        } else if let Some(else_branch) = else_branch {
            else_branch.accept(self)?;
        }
        Ok(())
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> InterpreterResult<()> {
        while condition.accept(self)?.is_truthy() {
            body.accept(self)?;
        }
        Ok(())
    }

    fn visit_return(&mut self, value: Option<&Expr>) -> InterpreterResult<()> {
        let value = value
            .map(|expr| expr.accept(self))
            .transpose()?
            .unwrap_or(Value::Nil);
        return Err(RuntimeControl::Return(value));
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> InterpreterResult<()> {
        self.enter_scope();
        for stmt in statements {
            stmt.accept(self)?;
        }
        self.exit_scope();
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
            BinaryOp::Add => left + right,
            BinaryOp::Subtract => left - right,
            BinaryOp::Multiply => left * right,
            BinaryOp::Divide => left / right,
            BinaryOp::Equal => Ok(Value::Boolean(left == right)),
            BinaryOp::NotEqual => Ok(Value::Boolean(left != right)),
            BinaryOp::LessThan => Ok(Value::Boolean(left < right)),
            BinaryOp::LessEqual => Ok(Value::Boolean(left <= right)),
            BinaryOp::GreaterThan => Ok(Value::Boolean(left > right)),
            BinaryOp::GreaterEqual => Ok(Value::Boolean(left >= right)),
            BinaryOp::LogicalAnd => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
            BinaryOp::LogicalOr => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),
        }
    }

    fn visit_assign(&mut self, name: &str, value: &Expr) -> Result<Value> {
        let value = value.accept(self)?;
        self.env.borrow_mut().set(name, value.clone())?;
        Ok(value)
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> Result<Value> {
        let callee = callee.accept(self)?;
        let arguments = arguments
            .iter()
            .map(|arg| arg.accept(self))
            .collect::<Result<Vec<Value>>>()?;

        match callee {
            Value::Function {
                params,
                body,
                closure,
                ..
            } => {
                let prev_env = Rc::clone(&self.env);
                self.env = Environment::new_local(&closure);

                for (param, arg) in params.iter().zip(arguments.iter()) {
                    self.env.borrow_mut().define(param.clone(), arg.clone());
                }
                let result = body.accept(self);

                self.env = prev_env;
                match result {
                    Ok(_) => Ok(Value::Nil),
                    Err(RuntimeControl::Return(value)) => Ok(value),
                    Err(RuntimeControl::Error(e)) => Err(e),
                }
            }
            _ => Err(Error::runtime(format!(
                "can only call functions. Got {callee}"
            ))),
        }
    }

    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Value> {
        let operand = operand.accept(self)?;
        match op {
            UnaryOp::Minus => -operand,
            UnaryOp::Not => Ok(Value::Boolean(!operand.is_truthy())),
        }
    }
}
