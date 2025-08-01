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

    fn number_operands_error(op: &BinaryOp, left: &Value, right: &Value) -> Error {
        Error::runtime(format!(
            "Operands must be numbers. Got {left:#?} and {right:?} for {op:?}"
        ))
    }
}

impl stmt::Visitor<InterpreterResult<()>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> InterpreterResult<()> {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_print(&mut self, expr: &Expr) -> InterpreterResult<()> {
        let value = expr.accept(self)?;
        println!("{value}");
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
            closure: self.env.clone(),
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
            BinaryOp::Add => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                    _ => Err(Error::runtime(format!("Operands must be numbers or strings. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::Subtract => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                    _ => Err(Self::number_operands_error(op, &left, &right)),
                }
            },
            BinaryOp::Multiply => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                    _ => Err(Self::number_operands_error(op, &left, &right)),
                }
            },
            BinaryOp::Divide => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                    _ => Err(Self::number_operands_error(op, &left, &right)),
                }
            },
            BinaryOp::Equal => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),
                    (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
                    _ => Err(Error::runtime(format!("Operands must be numbers, strings or booleans. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::NotEqual => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a != b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a != b)),
                    (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a != b)),
                    _ => Err(Error::runtime(format!("Operands must be numbers, strings or booleans. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::LessThan => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
                    _ => Err(Error::runtime(format!("Operands must be numbers or strings. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::LessEqual => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
                    _ => Err(Error::runtime(format!("Operands must be numbers or strings. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::GreaterThan => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
                    _ => Err(Error::runtime(format!("Operands must be numbers or strings. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::GreaterEqual => {
                match (left.clone(), right.clone()) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
                    _ => Err(Error::runtime(format!("Operands must be numbers or strings. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::LogicalAnd => {
                match (left.clone(), right.clone()) {
                    (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a && b)),
                    _ => Err(Error::runtime(format!("Operands must be booleans. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
            BinaryOp::LogicalOr => {
                match (left.clone(), right.clone()) {
                    (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a || b)),
                    _ => Err(Error::runtime(format!("Operands must be booleans. Got {left:#?} and {right:?} for {op:?}"))),
                }
            },
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
                let local_env = Environment::new_local(&closure);
                for (param, arg) in params.iter().zip(arguments) {
                    local_env.borrow_mut().define(param.clone(), arg.clone());
                }
                match body.accept(self) {
                    Ok(_) => Ok(Value::Nil),
                    Err(RuntimeControl::Return(value)) => Ok(value),
                    Err(RuntimeControl::Error(e)) => Err(e),
                }
            }
            _ => Err(Error::runtime(format!(
                "Can only call functions. Got {callee:?}"
            ))),
        }
    }

    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Value> {
        let operand = operand.accept(self)?;
        match op {
            UnaryOp::Minus => match operand {
                Value::Number(a) => Ok(Value::Number(-a)),
                _ => Err(Error::runtime(format!(
                    "Operand must be a number. Got {operand:?} for {op:?}"
                ))),
            },
            UnaryOp::Not => match operand {
                Value::Boolean(a) => Ok(Value::Boolean(!a)),
                _ => Err(Error::runtime(format!(
                    "Operand must be a boolean. Got {operand:?} for {op:?}"
                ))),
            },
        }
    }
}
