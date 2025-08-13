use crate::{
    compiler::value::{Function, Upvalue},
    constant::STACK_SIZE,
    error::{Error, Result},
};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: Rc<Function>,
    pub ip: usize,
    pub slots_offset: usize,
}

pub struct CallStack {
    frames: Vec<CallFrame>,
}

impl CallStack {
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(STACK_SIZE),
        }
    }

    pub fn push(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }

    pub fn offset(&self) -> usize {
        self.frames
            .last()
            .map(|frame| frame.slots_offset)
            .unwrap_or(0)
    }

    pub fn get_upvalue(&self, index: usize) -> Result<&Upvalue> {
        if let Some(frame) = self.frames.last() {
            let function = &frame.function;
            if index < function.upvalues.len() {
                Ok(&function.upvalues[index])
            } else {
                Err(Error::upvalue_index_out_of_bounds(
                    index,
                    function.upvalues.len(),
                ))
            }
        } else {
            Err(Error::runtime(
                "Cannot get upvalue from empty call stack".to_string(),
            ))
        }
    }
}
