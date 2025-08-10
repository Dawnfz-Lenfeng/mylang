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
use std::{io::Write, rc::Rc};

pub struct Interpreter {
    env: EnvRef,
    output: Box<dyn Write>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new_global(),
            output: Box::new(std::io::stdout()),
        }
    }

    pub fn with_output(output: Box<dyn Write>) -> Self {
        Self {
            env: Environment::new_global(),
            output,
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            stmt.accept(self)?;
        }
        Ok(())
    }

    pub fn begin_scope(&mut self) {
        self.env = Environment::new_enclosed(self.env.clone());
    }

    pub fn end_scope(&mut self) {
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

        writeln!(self.output, "{output}")
            .map_err(|e| RuntimeControl::Error(Error::io(e.to_string())))?;
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
        body: &[Stmt],
    ) -> InterpreterResult<()> {
        let func = Value::Function {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.to_vec(),
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
            match body.accept(self) {
                Ok(_) => (),
                Err(RuntimeControl::Break) => break,
                Err(RuntimeControl::Continue) => (),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    fn visit_break(&mut self) -> InterpreterResult<()> {
        Err(RuntimeControl::Break)
    }

    fn visit_continue(&mut self) -> InterpreterResult<()> {
        Err(RuntimeControl::Continue)
    }

    fn visit_return(&mut self, value: Option<&Expr>) -> InterpreterResult<()> {
        let value = value
            .map(|expr| expr.accept(self))
            .transpose()?
            .unwrap_or(Value::Nil);
        return Err(RuntimeControl::Return(value));
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> InterpreterResult<()> {
        self.begin_scope();
        for stmt in statements {
            stmt.accept(self)?;
        }
        self.end_scope();
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

    fn visit_nil(&mut self) -> Result<Value> {
        Ok(Value::Nil)
    }

    fn visit_identifier(&mut self, name: &str) -> Result<Value> {
        self.env.borrow().get(name)
    }

    fn visit_array(&mut self, elements: &[Expr]) -> Result<Value> {
        let values = elements
            .iter()
            .map(|expr| expr.accept(self))
            .collect::<Result<Vec<Value>>>()?;
        Ok(Value::Array(values))
    }

    fn visit_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<Value> {
        match op {
            BinaryOp::LogicalAnd => {
                let left = left.accept(self)?;
                if !left.is_truthy() {
                    Ok(Value::Boolean(false))
                } else {
                    let right = right.accept(self)?;
                    Ok(Value::Boolean(right.is_truthy()))
                }
            }
            BinaryOp::LogicalOr => {
                let left = left.accept(self)?;
                if left.is_truthy() {
                    Ok(Value::Boolean(true))
                } else {
                    let right = right.accept(self)?;
                    Ok(Value::Boolean(right.is_truthy()))
                }
            }
            _ => {
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
                    _ => unreachable!(),
                }
            }
        }
    }

    fn visit_assign(&mut self, name: &str, value: &Expr) -> Result<Value> {
        let value = value.accept(self)?;
        self.env.borrow_mut().set(name, value.clone())?;
        Ok(value)
    }

    fn visit_index(&mut self, array: &Expr, index: &Expr) -> Result<Value> {
        let array_value = array.accept(self)?;
        let index_value = index.accept(self)?;

        match (array_value, index_value) {
            (Value::Array(arr), Value::Number(idx)) => {
                let idx = idx as usize;
                if idx < arr.len() {
                    Ok(arr[idx].clone())
                } else {
                    Err(Error::runtime(format!(
                        "Array index {} out of bounds (length: {})",
                        idx,
                        arr.len()
                    )))
                }
            }
            (Value::Array(_), _) => Err(Error::runtime("array index must be a number".to_string())),
            _ => Err(Error::runtime("cannot index non-array value".to_string())),
        }
    }

    fn visit_index_assign(&mut self, array: &Expr, index: &Expr, value: &Expr) -> Result<Value> {
        let index_value = index.accept(self)?;
        let new_value = value.accept(self)?;

        match index_value {
            Value::Number(idx) => {
                let idx = idx as usize;
                match array {
                    Expr::Variable(name) => {
                        let mut array_value = self.env.borrow().get(name)?;
                        match &mut array_value {
                            Value::Array(ref mut arr) => {
                                if idx < arr.len() {
                                    arr[idx] = new_value.clone();
                                    self.env.borrow_mut().set(name, array_value)?;
                                    Ok(new_value)
                                } else {
                                    Err(Error::runtime(format!(
                                        "Array index {} out of bounds (length: {})",
                                        idx,
                                        arr.len()
                                    )))
                                }
                            }
                            _ => Err(Error::runtime(
                                "Cannot index assign to non-array value".to_string(),
                            )),
                        }
                    }
                    _ => Err(Error::runtime(
                        "Can only assign to array variables".to_string(),
                    )),
                }
            }
            _ => Err(Error::runtime("Array index must be a number".to_string())),
        }
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
                if params.len() != arguments.len() {
                    return Err(Error::runtime(format!(
                        "Expected {} arguments, got {}",
                        params.len(),
                        arguments.len()
                    )));
                }
                let prev_env = Rc::clone(&self.env);
                self.env = Environment::new_enclosed(closure);

                for (param, arg) in params.iter().zip(arguments.iter()) {
                    self.env.borrow_mut().define(param.clone(), arg.clone());
                }
                let result = body.iter().try_for_each(|stmt| stmt.accept(self));

                self.env = prev_env;

                match result {
                    Ok(_) => Ok(Value::Nil),
                    Err(RuntimeControl::Return(value)) => Ok(value),
                    Err(e) => Err(e.into()),
                }
            }
            Value::BuiltinFunction { function, .. } => function(&arguments),
            _ => Err(Error::runtime(format!(
                "can only call functions. Got {callee}"
            ))),
        }
    }

    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Value> {
        let operand = operand.accept(self)?;
        match op {
            UnaryOp::Negate => -operand,
            UnaryOp::Not => Ok(Value::Boolean(!operand.is_truthy())),
        }
    }
}
