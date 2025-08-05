use crate::{
    error::{Error, Result},
    parser::Stmt,
};
use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Function {
        name: String,
        params: Vec<String>,
        body: Stmt,
    },
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Function { .. } => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Function { .. } => "function",
            Value::Nil => "nil",
        }
    }
}

impl Add for Value {
    type Output = Result<Value>;

    fn add(self, other: Self) -> Self::Output {
        let self_type = self.type_name();
        let other_type = other.type_name();

        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (Value::Array(a), Value::Array(b)) => {
                Ok(Value::Array(a.iter().chain(b.iter()).cloned().collect()))
            }
            _ => Err(Error::runtime(format!(
                "unsupported operand type(s) for +: '{self_type}' and '{other_type}'"
            ))),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value>;

    fn sub(self, other: Self) -> Self::Output {
        let self_type = self.type_name();
        let other_type = other.type_name();

        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(Error::runtime(format!(
                "unsupported operand type(s) for -: '{self_type}' and '{other_type}'"
            ))),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value>;

    fn mul(self, other: Self) -> Self::Output {
        let self_type = self.type_name();
        let other_type = other.type_name();

        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(Error::runtime(format!(
                "unsupported operand type(s) for *: '{self_type}' and '{other_type}'"
            ))),
        }
    }
}

impl Div for Value {
    type Output = Result<Value>;

    fn div(self, other: Self) -> Self::Output {
        let self_type = self.type_name();
        let other_type = other.type_name();

        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
            _ => Err(Error::runtime(format!(
                "unsupported operand type(s) for /: '{self_type}' and '{other_type}'"
            ))),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Function { name: a_name, .. }, Value::Function { name: b_name, .. }) => {
                a_name == b_name
            }
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Array(a), Value::Array(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Neg for Value {
    type Output = Result<Value>;

    fn neg(self) -> Self::Output {
        let self_type = self.type_name();
        match self {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(Error::runtime(format!(
                "bad operand type for unary -: '{self_type}'"
            ))),
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
            Value::Nil => write!(f, "nil"),
        }
    }
}
