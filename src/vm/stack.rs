use crate::{
    compliler::value::{Closure, Upvalue},
    error::{Error, Result},
};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub closure: Rc<Closure>,
    pub ip: usize,
    pub slots_offset: usize,
}

pub struct CallStack {
    frames: Vec<CallFrame>,
}

impl CallStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn push(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }

    pub fn peek(&self) -> Option<&CallFrame> {
        self.frames.last()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn offset(&self) -> usize {
        self.frames
            .last()
            .map(|frame| frame.slots_offset)
            .unwrap_or(0)
    }

    pub fn get_upvalue(&self, index: usize) -> Result<&Upvalue> {
        if let Some(frame) = self.frames.last() {
            let closure = &frame.closure;
            if index < closure.upvalues.len() {
                Ok(&closure.upvalues[index])
            } else {
                Err(Error::upvalue_index_out_of_bounds(
                    index,
                    closure.upvalues.len(),
                ))
            }
        } else {
            Err(Error::runtime(
                "Cannot get upvalue from empty call stack".to_string(),
            ))
        }
    }
}
