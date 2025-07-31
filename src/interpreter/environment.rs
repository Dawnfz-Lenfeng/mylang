use crate::error::runtime_error::RuntimeError;
use super::value::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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