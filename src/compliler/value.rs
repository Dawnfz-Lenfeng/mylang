use crate::error::{Error, Result};
use std::{
    cell::RefCell,
    cmp::Ordering,
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

pub type Upvalue = Rc<RefCell<Value>>;

#[derive(Debug, Clone, PartialEq)]
pub struct UpvalueInfo {
    pub index: usize,
    /// true if upvalue refers to local variable, false if it refers to upvalue
    pub is_local: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    pub name: String,
    pub params: Vec<String>,
    pub start_ip: usize,
    pub upvalues: Vec<UpvalueInfo>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub start_ip: usize,
    pub upvalues: Vec<Upvalue>,
}

impl Function {
    pub fn from_proto(proto: Proto, upvalues: Vec<Upvalue>) -> Self {
        Self {
            name: proto.name,
            params: proto.params,
            start_ip: proto.start_ip,
            upvalues,
        }
    }
    pub fn upvalue_count(&self) -> usize {
        self.upvalues.len()
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Rc<RefCell<Vec<Value>>>),
    Proto(Proto),
    Function(Rc<Function>),
    Nil,
}

impl Value {
    pub fn new_upvalue(value: Value) -> Upvalue {
        Rc::new(RefCell::new(value))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.borrow().is_empty(),
            Value::Proto(_) => true,
            Value::Function(_) => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Proto(_) => "proto",
            Value::Function(_) => "function",
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
            (Value::Array(a), Value::Array(b)) => Ok(Value::Array(Rc::new(RefCell::new(
                a.borrow()
                    .iter()
                    .chain(b.borrow().iter())
                    .cloned()
                    .collect(),
            )))),
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
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
            (Value::Proto(a), Value::Proto(b)) => a == b,
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
            (Value::Array(a), Value::Array(b)) => a.borrow().partial_cmp(&b.borrow()),
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
                for (i, val) in arr.borrow().iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::Proto(function) => {
                write!(
                    f,
                    "<proto {}({}) upvals:{}>",
                    function.name,
                    function.params.join(", "),
                    function.upvalues.len()
                )
            }
            Value::Function(function) => {
                write!(
                    f,
                    "<function {}> upvals:{}",
                    function.name,
                    function.upvalue_count()
                )
            }
            Value::Nil => write!(f, "nil"),
        }
    }
}
