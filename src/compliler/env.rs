use super::value::UpvalueInfo;
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: usize,
    pub is_captured: bool,
}
use std::{cell::RefCell, rc::Rc};

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Debug)]
pub struct LoopContext {
    pub break_jumps: Vec<usize>,
    pub continue_jumps: Vec<usize>,
}

#[derive(Debug)]
pub struct Env {
    pub locals: Vec<Local>,
    pub upvalues: Vec<UpvalueInfo>,
    pub scope_depth: usize,
    pub enclosing: Option<EnvRef>,
    pub loop_contexts: Vec<LoopContext>,
}

impl Env {
    pub fn new_global() -> EnvRef {
        Rc::new(RefCell::new(Self {
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            enclosing: None,
            loop_contexts: Vec::new(),
        }))
    }

    pub fn new_enclosed(enclosing: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: enclosing.borrow().scope_depth + 1,
            enclosing: Some(enclosing.clone()),
            loop_contexts: Vec::new(),
        }))
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Returns the number of locals to pop
    pub fn end_scope(&mut self) -> Result<usize> {
        if self.is_global() {
            return Err(Error::quit_from_global());
        }
        self.scope_depth -= 1;

        let mut pop_count = 0;
        while self
            .locals
            .last()
            .map(|l| l.depth > self.scope_depth)
            .unwrap_or(false)
        {
            self.locals.pop();
            pop_count += 1;
        }

        Ok(pop_count)
    }

    pub fn is_global(&self) -> bool {
        self.scope_depth == 0
    }

    pub fn add_locals(&mut self, names: &[String]) {
        for name in names {
            self.add_local(name.clone());
        }
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
            let mut enc = self.enclosing.as_ref()?.borrow_mut();

            if let Some(local_index) = enc.resolve_local(name) {
                enc.locals[local_index as usize].is_captured = true;
                (local_index, true)
            } else {
                (enc.resolve_upvalue(name)?, false)
            }
        };

        Some(self.add_upvalue(index as usize, is_local))
    }

    pub fn begin_loop(&mut self) {
        self.loop_contexts.push(LoopContext {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });
    }

    pub fn end_loop(&mut self) -> Option<LoopContext> {
        self.loop_contexts.pop()
    }

    pub fn add_break_jump(&mut self, jump_position: usize) -> Result<()> {
        if let Some(context) = self.loop_contexts.last_mut() {
            context.break_jumps.push(jump_position);
            Ok(())
        } else {
            Err(Error::runtime("break outside of loop".to_string()))
        }
    }

    pub fn add_continue_jump(&mut self, jump_position: usize) -> Result<()> {
        if let Some(context) = self.loop_contexts.last_mut() {
            context.continue_jumps.push(jump_position);
            Ok(())
        } else {
            Err(Error::runtime("continue outside of loop".to_string()))
        }
    }

    pub fn in_loop(&self) -> bool {
        !self.loop_contexts.is_empty()
    }
}
