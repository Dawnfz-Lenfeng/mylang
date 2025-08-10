use super::{buildin::BUILTIN_FUNCTIONS, value::Value};
use crate::error::{Error, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    pub enclosing: Option<EnvRef>,
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new_global() -> EnvRef {
        let mut variables = HashMap::new();

        for (name, func) in BUILTIN_FUNCTIONS {
            let builtin_value = Value::BuiltinFunction {
                name: name.to_string(),
                function: *func,
            };
            variables.insert(name.to_string(), builtin_value);
        }

        Rc::new(RefCell::new(Environment {
            enclosing: None,
            variables,
        }))
    }

    pub fn new_enclosed(enclosing: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing),
            variables: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.variables.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = &self.enclosing {
            return parent.borrow().get(name);
        }

        Err(Error::runtime(format!("name '{name}' is not defined")))
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().set(name, value);
        }

        Err(Error::runtime(format!("name '{name}' is not defined")))
    }
}
