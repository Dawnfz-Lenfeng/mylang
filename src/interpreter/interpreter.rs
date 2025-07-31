use crate::error::error::CompilerError;
use crate::error::runtime_error::RuntimeError;
use crate::interpreter::environment::{EnvRef, Environment};
use crate::interpreter::value::{Value, NULL};
use crate::parser::{
    expr::{BinaryOp, Expr, UnaryOp},
    stmt::{Parameter, Program, Stmt},
};
use std::rc::Rc;

/// Main interpreter that executes the AST
pub struct Interpreter {
    current_env: EnvRef,
}

impl Interpreter {
    /// Create a new interpreter instance
    pub fn new() -> Self {
        let global_env = Environment::new().into_rc_ref();

        // Register built-in functions in global environment
        global_env.borrow_mut().define(
            "print".to_string(),
            Value::Function {
                name: "print".to_string(),
                params: vec!["...args".to_string()],
                body: vec![], // Empty body for built-ins
                closure: Rc::clone(&global_env),
            },
        );

        global_env.borrow_mut().define(
            "len".to_string(),
            Value::Function {
                name: "len".to_string(),
                params: vec!["obj".to_string()],
                body: vec![],
                closure: Rc::clone(&global_env),
            },
        );

        global_env.borrow_mut().define(
            "type".to_string(),
            Value::Function {
                name: "type".to_string(),
                params: vec!["obj".to_string()],
                body: vec![],
                closure: Rc::clone(&global_env),
            },
        );

        Self {
            current_env: global_env,
        }
    }

    pub fn enter_scope(&mut self) {
        let new_env = Environment::new_child(&self.current_env);
        self.current_env = new_env.into_rc_ref();
    }

    pub fn exit_scope(&mut self) {
        let parent = self.current_env.borrow_mut().parent.take();
        self.current_env = parent.unwrap();
    }

    /// Execute a program
    pub fn interpret(&mut self, program: &Program) -> Result<Value, CompilerError> {
        let mut result = NULL;

        for (i, statement) in program.statements.iter().enumerate() {
            let is_last = i == program.statements.len() - 1;

            match statement {
                Stmt::Expression(expr) => match self.evaluate_expression(expr) {
                    Ok(val) => {
                        if is_last {
                            result = val;
                        }
                    }
                    Err(runtime_err) => {
                        return self.convert_runtime_error(runtime_err);
                    }
                },
                _ => match self.execute_statement(statement) {
                    Ok(()) => {}
                    Err(runtime_err) => {
                        return self.convert_runtime_error(runtime_err);
                    }
                },
            }
        }

        Ok(result)
    }

    /// Convert RuntimeError to CompilerError for public API
    fn convert_runtime_error(&self, runtime_err: RuntimeError) -> Result<Value, CompilerError> {
        match runtime_err {
            RuntimeError::Error { message } => Err(CompilerError::runtime_error(message)),
            RuntimeError::Return { .. } => Err(CompilerError::runtime_error(
                "return statement outside function".to_string(),
            )),
            _ => Err(CompilerError::runtime_error(format!(
                "Unexpected control flow: {}",
                runtime_err
            ))),
        }
    }

    /// Execute a single statement
    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => self.execute_var_declaration(name, initializer),
            Stmt::FuncDecl {
                name,
                parameters,
                body,
                ..
            } => self.execute_function_declaration(name, parameters, body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.execute_if_statement(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.execute_while_statement(condition, body),
            Stmt::For {
                name,
                collection,
                body,
            } => self.execute_for_statement(name, collection, body),
            Stmt::Return { value } => self.execute_return_statement(value),
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(())
            }
            Stmt::Block(statements) => self.execute_block_statement(statements),
        }
    }

    fn execute_var_declaration(
        &mut self,
        name: &str,
        initializer: &Option<Expr>,
    ) -> Result<(), RuntimeError> {
        if let Some(initializer) = initializer {
            let value = self.evaluate_expression(initializer)?;
            self.current_env
                .borrow_mut()
                .define(name.to_string(), value.clone());
        } else {
            self.current_env.borrow_mut().define(name.to_string(), NULL);
        }
        Ok(())
    }

    fn execute_function_declaration(
        &mut self,
        name: &str,
        parameters: &[Parameter],
        body: &[Stmt],
    ) -> Result<(), RuntimeError> {
        let function = Value::Function {
            name: name.to_string(),
            params: parameters.iter().map(|p| p.name.clone()).collect(),
            body: body.to_vec(),
            closure: Rc::clone(&self.current_env),
        };

        self.current_env
            .borrow_mut()
            .define(name.to_string(), function.clone());

        Ok(())
    }

    fn execute_if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &Option<Vec<Stmt>>,
    ) -> Result<(), RuntimeError> {
        if self.evaluate_expression(condition)?.is_truthy() {
            self.execute_block_statement(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute_block_statement(else_branch)?;
        }
        Ok(())
    }

    fn execute_while_statement(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
    ) -> Result<(), RuntimeError> {
        while self.evaluate_expression(condition)?.is_truthy() {
            self.execute_block_statement(body)?;
        }
        Ok(())
    }

    fn execute_for_statement(
        &mut self,
        _name: &str,
        _collection: &Expr,
        _body: &[Stmt],
    ) -> Result<(), RuntimeError> {
        // TODO: Implement for statement
        Ok(())
    }

    fn execute_return_statement(&mut self, value: &Option<Expr>) -> Result<(), RuntimeError> {
        let return_value = if let Some(expr) = value {
            self.evaluate_expression(expr)?
        } else {
            NULL
        };

        Err(RuntimeError::return_value(return_value))
    }

    fn execute_block_statement(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        self.enter_scope();

        for statement in statements {
            match self.execute_statement(statement) {
                Ok(()) => {}
                Err(err) => {
                    self.exit_scope();
                    return Err(err);
                }
            }
        }

        self.exit_scope();
        Ok(())
    }

    /// Evaluate an expression and return its value
    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Variable(name) => self.current_env.borrow().get(name),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary_expression(left, operator, right),
            Expr::Unary { operator, operand } => self.evaluate_unary_expression(operator, operand),
            Expr::Call { callee, arguments } => self.evaluate_call_expression(callee, arguments),
            Expr::Index { array, index } => self.evaluate_index_expression(array, index),
            Expr::Assign { name, value } => self.evaluate_assignment(name, value),
            Expr::Array { elements } => self.evaluate_array_expression(elements),
        }
    }

    /// Evaluate binary expressions (+, -, *, /, etc.)
    fn evaluate_binary_expression(
        &mut self,
        left: &Expr,
        operator: &BinaryOp,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        let left_value = self.evaluate_expression(left)?;
        let right_value = self.evaluate_expression(right)?;

        match operator {
            BinaryOp::Add => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => {
                    Ok(Value::String(left.clone() + &right))
                }
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for addition: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::Subtract => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for subtraction: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::Multiply => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for multiplication: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::Divide => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for division: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::Modulo => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left % right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for modulo: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::Equal => Ok(Value::Boolean(left_value == right_value)),
            BinaryOp::LessThan => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for less than: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::LessEqual => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for less than or equal: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::GreaterThan => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for greater than: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            BinaryOp::GreaterEqual => match (&left_value, &right_value) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
                _ => Err(RuntimeError::error(format!(
                    "Invalid types for greater than or equal: {:?} and {:?}",
                    left_value, right_value
                ))),
            },
            _ => Err(RuntimeError::error(format!(
                "Binary operation not implemented: {:?}",
                operator
            ))),
        }
    }

    /// Evaluate unary expressions (-, not)
    fn evaluate_unary_expression(
        &mut self,
        operator: &UnaryOp,
        operand: &Expr,
    ) -> Result<Value, RuntimeError> {
        // TODO: Implement unary operations

        Err(RuntimeError::error(
            "Unary operations not implemented".to_string(),
        ))
    }

    /// Evaluate function calls
    fn evaluate_call_expression(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
    ) -> Result<Value, RuntimeError> {
        let callee_value = self.evaluate_expression(callee)?;
        let arg_values = arguments
            .iter()
            .map(|arg| self.evaluate_expression(arg))
            .collect::<Result<Vec<_>, _>>()?;

        match callee_value {
            Value::Function {
                name,
                params,
                body,
                closure,
            } => match name.as_str() {
                "print" => {
                    return builtin_print(&arg_values);
                }
                "len" => {
                    return builtin_len(&arg_values);
                }
                "type" => {
                    return builtin_type(&arg_values);
                }
                _ => {
                    if params.len() != arg_values.len() {
                        return Err(RuntimeError::error(format!(
                            "Function '{}' expects {} arguments, got {}",
                            name,
                            params.len(),
                            arg_values.len()
                        )));
                    }

                    let previous_env = Rc::clone(&self.current_env);
                    let mut function_env = Environment::new_child(&closure);

                    for (param, arg_value) in params.iter().zip(arg_values.iter()) {
                        function_env.define(param.clone(), arg_value.clone());
                    }

                    self.current_env = function_env.into_rc_ref();

                    let result = match self.execute_block_statement(&body) {
                        Ok(()) => NULL, // 函数没有明确return，返回NULL
                        Err(runtime_err) if runtime_err.is_return() => {
                            let return_value = runtime_err.get_return_value().unwrap_or(NULL);
                            self.current_env = previous_env;
                            return Ok(return_value);
                        }
                        Err(err) => {
                            self.current_env = previous_env;
                            return Err(err);
                        }
                    };

                    self.current_env = previous_env;
                    Ok(result)
                }
            },
            _ => Err(RuntimeError::error(format!(
                "'{:?}' is not a function",
                callee_value
            ))),
        }
    }

    /// Evaluate array indexing
    fn evaluate_index_expression(
        &mut self,
        array: &Expr,
        index: &Expr,
    ) -> Result<Value, RuntimeError> {
        // TODO: Implement array indexing

        Err(RuntimeError::error(
            "Array indexing not implemented".to_string(),
        ))
    }

    /// Evaluate assignment expressions
    fn evaluate_assignment(&mut self, name: &str, value: &Expr) -> Result<Value, RuntimeError> {
        let evaluated_value = self.evaluate_expression(value)?;
        self.current_env
            .borrow_mut()
            .set(name, evaluated_value.clone())?;
        Ok(evaluated_value)
    }

    /// Evaluate array expressions
    fn evaluate_array_expression(&mut self, elements: &[Expr]) -> Result<Value, RuntimeError> {
        // TODO: Implement array expressions

        Err(RuntimeError::error(
            "Array expressions not implemented".to_string(),
        ))
    }
}

// Built-in function implementations

/// Built-in print function
fn builtin_print(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(NULL)
}

/// Built-in len function
fn builtin_len(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::error(
            "len() takes exactly 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
        _ => Err(RuntimeError::error(
            "len() argument must be string or array".to_string(),
        )),
    }
}

/// Built-in type function
fn builtin_type(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::error(
            "type() takes exactly 1 argument".to_string(),
        ));
    }

    Ok(Value::String(args[0].type_name().to_string()))
}
