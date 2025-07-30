use crate::ast::Stmt;
use crate::interpreter::error::RuntimeError;
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

pub type EnvRef = Rc<RefCell<Environment>>;

/// Runtime environment for variable storage and scoping
#[derive(Debug)]
pub struct Environment {
    pub parent: Option<EnvRef>,
    variables: HashMap<String, Value>,
}

impl Environment {
    /// Create a new environment with global scope
    pub fn new() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }

    pub fn new_child(parent: &EnvRef) -> Self {
        Self {
            parent: Some(Rc::clone(parent)),
            variables: HashMap::new(),
        }
    }

    /// Define a variable in current scope
    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Get variable value from any accessible scope
    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        if let Some(value) = self.variables.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        Err(RuntimeError::error(format!(
            "Undefined variable '{}'",
            name
        )))
    }

    /// Set variable value (must already exist)
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set(name, value);
        }

        Err(RuntimeError::error(format!(
            "Undefined variable '{}'",
            name
        )))
    }

    pub fn into_rc_ref(self) -> EnvRef {
        Rc::new(RefCell::new(self))
    }
}

pub const NULL: Value = Value::Null;

/// Runtime value types that the interpreter can work with
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: EnvRef, // Capture lexical scope
    },
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (
                Value::Function {
                    name: a_name,
                    closure: a_closure,
                    ..
                },
                Value::Function {
                    name: b_name,
                    closure: b_closure,
                    ..
                },
            ) => Rc::ptr_eq(a_closure, b_closure) && a_name == b_name,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
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
