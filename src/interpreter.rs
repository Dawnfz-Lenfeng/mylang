use crate::ast::{BinaryOp, Expr, Parameter, Program, Stmt, UnaryOp};
use crate::error::CompilerError;
use std::collections::HashMap;
use std::fmt;

/// Runtime value types that the interpreter can work with
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: Environment, // Capture lexical scope
    },
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::Function { name, params, .. } => {
                write!(f, "<function {}({})>", name, params.join(", "))
            }
            Value::Null => write!(f, "null"),
        }
    }
}

impl Value {
    /// Check if value is truthy for conditional expressions
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Function { .. } => true,
        }
    }

    /// Get the type name of the value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "num",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Function { .. } => "function",
            Value::Null => "null",
        }
    }
}

/// Runtime environment for variable storage and scoping
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    /// Create a new environment with global scope
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Create environment with captured variables (for closures)
    pub fn with_enclosing(enclosing: &Environment) -> Self {
        let mut env = Self::new();
        // Copy all scopes from enclosing environment
        env.scopes = enclosing.scopes.clone();
        env
    }

    /// Enter a new scope (for blocks, functions)
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable in current scope
    pub fn define(&mut self, name: String, value: Value) -> Result<(), CompilerError> {
        self.scopes.last_mut().unwrap().insert(name, value);
        Ok(())
    }

    /// Get variable value from any accessible scope
    pub fn get(&self, name: &str) -> Result<Value, CompilerError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(CompilerError::runtime_error(format!(
            "Undefined variable '{}'",
            name
        )))
    }

    /// Set variable value (must already exist)
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), CompilerError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(CompilerError::runtime_error(format!(
            "Undefined variable '{}'",
            name
        )))
    }
}

/// Main interpreter that executes the AST
pub struct Interpreter {
    /// Global environment
    environment: Environment,
    /// Built-in functions
    builtins: HashMap<String, fn(&[Value]) -> Result<Value, CompilerError>>,
}

impl Interpreter {
    /// Create a new interpreter instance
    pub fn new() -> Self {
        let mut interpreter = Self {
            environment: Environment::new(),
            builtins: HashMap::new(),
        };

        interpreter.register_builtins();
        interpreter
    }

    /// Register built-in functions
    fn register_builtins(&mut self) {
        self.builtins.insert("print".to_string(), builtin_print);
        self.builtins.insert("len".to_string(), builtin_len);
        self.builtins.insert("type".to_string(), builtin_type);
    }

    /// Execute a program
    pub fn interpret(&mut self, program: &Program) -> Result<Value, CompilerError> {
        let mut result = Value::Null;

        for statement in &program.statements {
            result = self.execute_statement(statement)?;
        }

        Ok(result)
    }

    /// Execute a single statement
    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<Value, CompilerError> {
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation: _,
                initializer,
                is_mutable: _,
            } => self.evaluate_var_declaration(name, initializer),
            Stmt::FuncDecl {
                name,
                parameters,
                return_type: _,
                body,
            } => self.evaluate_function_declaration(name, parameters, body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.evaluate_if_statement(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.evaluate_while_statement(condition, body),
            Stmt::For {
                name,
                collection,
                body,
            } => self.evaluate_for_statement(name, collection, body),
            Stmt::Return { value } => self.evaluate_return_statement(value),
            Stmt::Expression(expr) => self.evaluate_expression_statement(expr),
            Stmt::Block(statements) => self.evaluate_block_statement(statements),
        }
    }

    fn evaluate_var_declaration(
        &mut self,
        name: &str,
        initializer: &Option<Expr>,
    ) -> Result<Value, CompilerError> {
        if let Some(initializer) = initializer {
            let value = self.evaluate_expression(initializer)?;
            self.environment.define(name.to_string(), value.clone())?;
        } else {
            self.environment.define(name.to_string(), Value::Null)?;
        }
        Ok(Value::Null)
    }

    fn evaluate_function_declaration(
        &mut self,
        name: &str,
        parameters: &[Parameter],
        body: &[Stmt],
    ) -> Result<Value, CompilerError> {
        // TODO: Implement function declaration
        Ok(Value::Null)
    }

    fn evaluate_if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &Option<Vec<Stmt>>,
    ) -> Result<Value, CompilerError> {
        // TODO: Implement if statement
        Ok(Value::Null)
    }

    fn evaluate_while_statement(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
    ) -> Result<Value, CompilerError> {
        // TODO: Implement while statement
        Ok(Value::Null)
    }

    fn evaluate_for_statement(
        &mut self,
        name: &str,
        collection: &Expr,
        body: &[Stmt],
    ) -> Result<Value, CompilerError> {
        // TODO: Implement for statement
        Ok(Value::Null)
    }

    fn evaluate_return_statement(&mut self, value: &Option<Expr>) -> Result<Value, CompilerError> {
        // TODO: Implement return statement
        Ok(Value::Null)
    }

    fn evaluate_block_statement(&mut self, statements: &[Stmt]) -> Result<Value, CompilerError> {
        // TODO: Implement block statement
        Ok(Value::Null)
    }

    fn evaluate_expression_statement(&mut self, expr: &Expr) -> Result<Value, CompilerError> {
        self.evaluate_expression(expr)
    }

    /// Evaluate an expression and return its value
    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, CompilerError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Identifier(name) => self.environment.get(name),
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
    ) -> Result<Value, CompilerError> {
        // TODO: Implement binary operations
        // 1. Evaluate left and right operands
        // 2. Check types are compatible for the operation
        // 3. Perform the operation and return result

        Err(CompilerError::runtime_error(
            "Binary operations not implemented".to_string(),
        ))
    }

    /// Evaluate unary expressions (-, not)
    fn evaluate_unary_expression(
        &mut self,
        operator: &UnaryOp,
        operand: &Expr,
    ) -> Result<Value, CompilerError> {
        // TODO: Implement unary operations

        Err(CompilerError::runtime_error(
            "Unary operations not implemented".to_string(),
        ))
    }

    /// Evaluate function calls
    fn evaluate_call_expression(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
    ) -> Result<Value, CompilerError> {
        // TODO: Implement function calls
        // 1. Evaluate callee to get function
        // 2. Evaluate all arguments
        // 3. Check if it's a built-in function
        // 4. If user function, create new environment and execute body
        // 5. Handle return values properly

        Err(CompilerError::runtime_error(
            "Function calls not implemented".to_string(),
        ))
    }

    /// Evaluate array indexing
    fn evaluate_index_expression(
        &mut self,
        array: &Expr,
        index: &Expr,
    ) -> Result<Value, CompilerError> {
        // TODO: Implement array indexing

        Err(CompilerError::runtime_error(
            "Array indexing not implemented".to_string(),
        ))
    }

    /// Evaluate assignment expressions
    fn evaluate_assignment(&mut self, name: &str, value: &Expr) -> Result<Value, CompilerError> {
        // TODO: Implement assignment

        Err(CompilerError::runtime_error(
            "Assignment not implemented".to_string(),
        ))
    }

    /// Evaluate array expressions
    fn evaluate_array_expression(&mut self, elements: &[Expr]) -> Result<Value, CompilerError> {
        // TODO: Implement array expressions

        Err(CompilerError::runtime_error(
            "Array expressions not implemented".to_string(),
        ))
    }
}

// Built-in function implementations

/// Built-in print function
fn builtin_print(args: &[Value]) -> Result<Value, CompilerError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Null)
}

/// Built-in len function
fn builtin_len(args: &[Value]) -> Result<Value, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::runtime_error(
            "len() takes exactly 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
        _ => Err(CompilerError::runtime_error(
            "len() argument must be string or array".to_string(),
        )),
    }
}

/// Built-in type function
fn builtin_type(args: &[Value]) -> Result<Value, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::runtime_error(
            "type() takes exactly 1 argument".to_string(),
        ));
    }

    Ok(Value::String(args[0].type_name().to_string()))
}

/// Special exception type for handling early returns
#[derive(Debug)]
pub struct ReturnValue(pub Value);

impl fmt::Display for ReturnValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.0)
    }
}

impl std::error::Error for ReturnValue {}
