use crate::{
    compliler::{Chunk, Value},
};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: Value,
    pub ip: usize,
    pub slots_offset: usize,
    pub caller_chunk: Option<Chunk>,
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
        self.frames.last().map(|frame| frame.slots_offset).unwrap_or(0)
    }
}
