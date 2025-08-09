use super::value::UpvalueInfo;

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: usize,
    pub is_captured: bool,
}
use std::{cell::RefCell, rc::Rc};

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Debug)]
pub struct Env {
    pub locals: Vec<Local>,
    pub upvalues: Vec<UpvalueInfo>,
    pub scope_depth: usize,
    pub enclosing: Option<EnvRef>,
}

impl Env {
    pub fn new_global() -> EnvRef {
        Rc::new(RefCell::new(Self {
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            enclosing: None,
        }))
    }

    pub fn new_enclosed(enclosing: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: enclosing.borrow().scope_depth + 1,
            enclosing: Some(enclosing.clone()),
        }))
    }

    pub fn is_global(&self) -> bool {
        self.scope_depth == 0
    }

    pub fn add_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
            is_captured: false,
        });
    }

    pub fn resolve_local(&self, name: &str) -> Option<u8> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find(|(_, local)| local.name == name)
            .map(|(index, _)| index as u8)
    }

    pub fn add_upvalue(&mut self, index: usize, is_local: bool) -> u8 {
        for (i, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.index == index && upvalue.is_local == is_local {
                return i as u8;
            }
        }

        self.upvalues.push(UpvalueInfo { index, is_local });
        self.upvalues.len() as u8 - 1
    }

    pub fn resolve_upvalue(&mut self, name: &str) -> Option<u8> {
        let (index, is_local) = {
            let mut enc = self.enclosing.as_ref().map(|e| e.borrow_mut())?;

            if let Some(local_index) = enc.resolve_local(name) {
                enc.locals[local_index as usize].is_captured = true;
                (local_index, true)
            } else {
                (enc.resolve_upvalue(name)?, false)
            }
        };

        Some(self.add_upvalue(index as usize, is_local))
    }
}
