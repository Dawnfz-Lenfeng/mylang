use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
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
        // closure: Environment, // Capture lexical scope
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
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Function { .. } => "function",
            Value::Null => "null",
        }
    }
}

/// Runtime environment for variable storage and scoping
#[derive(Debug, Clone)]
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
        // TODO: Implement scope entry
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        // TODO: Implement scope exit
    }

    /// Define a variable in current scope
    pub fn define(&mut self, name: String, value: Value) -> Result<(), CompilerError> {
        // TODO: Implement variable definition
        Ok(())
    }

    /// Get variable value from any accessible scope
    pub fn get(&self, name: &str) -> Result<Value, CompilerError> {
        // TODO: Implement variable lookup
        Err(CompilerError::runtime_error(format!(
            "Undefined variable '{}'",
            name
        )))
    }

    /// Set variable value (must already exist)
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), CompilerError> {
        // TODO: Implement variable assignment
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
        // TODO: Register built-in functions like print, len, etc.
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
            } => {
                // TODO: Implement variable declaration
                Ok(Value::Null)
            }
            Stmt::FuncDecl {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                // TODO: Implement function declaration
                Ok(Value::Null)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // TODO: Implement if statement
                Ok(Value::Null)
            }
            Stmt::While { condition, body } => {
                // TODO: Implement while loop
                Ok(Value::Null)
            }
            Stmt::For {
                name,
                collection,
                body,
            } => {
                // TODO: Implement for loop
                Ok(Value::Null)
            }
            Stmt::Return { value } => {
                // TODO: Implement return statement
                // You might want to use a special exception/error type for early returns
                Ok(Value::Null)
            }
            Stmt::Expression(expr) => {
                // Execute expression and return its value
                self.evaluate_expression(expr)
            }
            Stmt::Block(statements) => {
                // TODO: Implement block execution with new scope
                Ok(Value::Null)
            }
        }
    }

    /// Evaluate an expression and return its value
    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, CompilerError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Identifier(name) => {
                // TODO: Look up variable in environment
                self.environment.get(name)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // TODO: Implement binary operations
                self.evaluate_binary_expression(left, operator, right)
            }
            Expr::Unary { operator, operand } => {
                // TODO: Implement unary operations
                self.evaluate_unary_expression(operator, operand)
            }
            Expr::Call { callee, arguments } => {
                // TODO: Implement function calls
                self.evaluate_call_expression(callee, arguments)
            }
            Expr::Index { array, index } => {
                // TODO: Implement array indexing
                self.evaluate_index_expression(array, index)
            }
            Expr::Assign { name, value } => {
                // TODO: Implement assignment
                self.evaluate_assignment(name, value)
            }
            Expr::Array { elements } => {
                // TODO: Implement array creation
                Ok(Value::Array(
                    elements
                        .iter()
                        .map(|e| self.evaluate_expression(e))
                        .collect::<Result<Vec<Value>, CompilerError>>()?,
                ))
            }
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
