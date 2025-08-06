use super::stack::{CallFrame, CallStack};
use crate::{
    compliler::{Chunk, OpCode, Value},
    error::{Error, Result},
};
use std::{collections::HashMap, io::Write};

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    call_stack: CallStack,
    output: Box<dyn Write>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: None,
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
            call_stack: CallStack::new(),
            output: Box::new(std::io::stdout()),
        }
    }

    pub fn with_output(output: Box<dyn Write>) -> Self {
        Self {
            output,
            ..Self::new()
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<()> {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.stack.clear();
        self.call_stack.clear();
        self.globals.clear();

        self.run()
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let chunk = self.chunk.as_ref().unwrap();
            if self.ip >= chunk.code.len() {
                break;
            }

            let instruction = OpCode::try_from(chunk.code[self.ip])?;
            self.ip += 1;

            match instruction {
                OpCode::Constant => {
                    let constant = self.read_constant()?;
                    self.push(constant);
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Boolean(true)),
                OpCode::False => self.push(Value::Boolean(false)),

                // Arithmetic
                OpCode::Add => self.binary_op(OpCode::Add)?,
                OpCode::Subtract => self.binary_op(OpCode::Subtract)?,
                OpCode::Multiply => self.binary_op(OpCode::Multiply)?,
                OpCode::Divide => self.binary_op(OpCode::Divide)?,
                OpCode::Negate => self.unary_op(OpCode::Negate)?,

                // Comparison
                OpCode::Equal => self.binary_op(OpCode::Equal)?,
                OpCode::NotEqual => self.binary_op(OpCode::NotEqual)?,
                OpCode::LessThan => self.binary_op(OpCode::LessThan)?,
                OpCode::LessEqual => self.binary_op(OpCode::LessEqual)?,
                OpCode::GreaterThan => self.binary_op(OpCode::GreaterThan)?,
                OpCode::GreaterEqual => self.binary_op(OpCode::GreaterEqual)?,

                // Logical
                OpCode::Not => self.unary_op(OpCode::Not)?,
                OpCode::And => self.binary_op(OpCode::And)?,
                OpCode::Or => self.binary_op(OpCode::Or)?,

                // Variables
                OpCode::DefineGlobal => {
                    let name = self.read_global_name()?;
                    let value = self.pop()?;
                    self.define_global(name, value);
                }
                OpCode::GetGlobal => {
                    let name = self.read_global_name()?;
                    let value = self.get_global(&name)?;
                    self.push(value);
                }
                OpCode::SetGlobal => {
                    let name = self.read_global_name()?;
                    let value = self.peek(0)?.clone();
                    self.set_global(name, value)?;
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte()? as usize;
                    let value = self.get_local(slot)?;
                    self.push(value);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte()? as usize;
                    let value = self.peek(0)?.clone();
                    self.set_local(slot, value)?;
                }

                // Control flow
                OpCode::Jump => {
                    let offset = self.read_short()? as usize;
                    self.ip += offset;
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short()? as usize;
                    let condition = self.peek(0)?;
                    if !condition.is_truthy() {
                        self.ip += offset;
                    }
                }
                OpCode::Loop => {
                    let offset = self.read_short()? as usize;
                    self.ip -= offset;
                }
                OpCode::Break => {
                    // Break implementation - handled by compiler
                    let offset = self.read_short()? as usize;
                    self.ip += offset;
                }
                OpCode::Continue => {
                    // Continue implementation - handled by compiler
                    let offset = self.read_short()? as usize;
                    self.ip -= offset;
                }

                // Functions
                OpCode::Call => {
                    let arg_count = self.read_byte()? as usize;
                    let callee = self.peek(arg_count)?.clone();
                    self.call_value(callee, arg_count)?;
                }
                OpCode::Return => {
                    let result = self.pop()?;
                    if let Some(frame) = self.call_stack.pop() {
                        self.ip = frame.ip;
                        self.stack.truncate(frame.slots_offset + 1); // +1 for function itself
                        self.push(result);
                        self.chunk = frame.caller_chunk;
                    } else {
                        return Ok(());
                    }
                }

                // Stack operations
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Print => self.print_values(1)?,

                // Arrays
                OpCode::Array => {
                    let element_count = self.read_byte()? as usize;
                    let array = self.create_array(element_count)?;
                    self.push(array);
                }
                OpCode::Index => {
                    self.index_array()?;
                }
                OpCode::IndexSet => {
                    self.set_array_element()?;
                }
            }
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let byte = self.chunk.as_ref().unwrap().code[self.ip];
        self.ip += 1;
        Ok(byte)
    }

    fn read_short(&mut self) -> Result<u16> {
        let byte1 = self.read_byte()? as u16;
        let byte2 = self.read_byte()? as u16;
        Ok((byte1 << 8) | byte2)
    }

    fn read_constant(&mut self) -> Result<Value> {
        let index = self.read_byte()?;
        Ok(self.chunk.as_ref().unwrap().constants[index as usize].clone())
    }

    fn read_global_name(&mut self) -> Result<String> {
        let index = self.read_byte()?;
        Ok(self.chunk.as_ref().unwrap().globals[index as usize].clone())
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::stack_underflow())
    }

    fn peek(&self, distance: usize) -> Result<&Value> {
        self.stack
            .get(self.stack.len() - distance - 1)
            .ok_or(Error::stack_underflow())
    }

    fn binary_op(&mut self, op: OpCode) -> Result<()> {
        let right = self.pop()?;
        let left = self.pop()?;

        match op {
            OpCode::Add => self.push((left + right)?),
            OpCode::Subtract => self.push((left - right)?),
            OpCode::Multiply => self.push((left * right)?),
            OpCode::Divide => self.push((left / right)?),
            OpCode::Equal => self.push(Value::Boolean(left == right)),
            OpCode::NotEqual => self.push(Value::Boolean(left != right)),
            OpCode::LessThan => self.push(Value::Boolean(left < right)),
            OpCode::LessEqual => self.push(Value::Boolean(left <= right)),
            OpCode::GreaterThan => self.push(Value::Boolean(left > right)),
            OpCode::GreaterEqual => self.push(Value::Boolean(left >= right)),
            OpCode::And => self.push(Value::Boolean(left.is_truthy() && right.is_truthy())),
            OpCode::Or => self.push(Value::Boolean(left.is_truthy() || right.is_truthy())),
            _ => return Err(Error::invalid_opcode(op as u8)),
        }
        Ok(())
    }

    fn unary_op(&mut self, op: OpCode) -> Result<()> {
        let operand = self.pop()?;
        match op {
            OpCode::Negate => self.push((-operand)?),
            OpCode::Not => self.push(Value::Boolean(!operand.is_truthy())),
            _ => return Err(Error::invalid_opcode(op as u8)),
        }
        Ok(())
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> Result<()> {
        match callee {
            Value::Function {
                name,
                params,
                chunk,
            } => {
                if params.len() != arg_count {
                    return Err(Error::arity_error(&name, params.len(), arg_count));
                }

                let frame = CallFrame {
                    function: Value::Function {
                        name,
                        params,
                        chunk: chunk.clone(),
                    },
                    ip: self.ip,
                    slots_offset: self.stack.len() - arg_count,
                    caller_chunk: self.chunk.clone(),
                };
                self.call_stack.push(frame);

                // Switch to function's chunk
                self.chunk = Some(*chunk);
                self.ip = 0;

                Ok(())
            }
            _ => Err(Error::runtime("Can only call functions".to_string())),
        }
    }

    fn get_global(&mut self, name: &str) -> Result<Value> {
        self.globals
            .get(name)
            .cloned()
            .ok_or(Error::undefined_variable(name))
    }

    fn set_global(&mut self, name: String, value: Value) -> Result<()> {
        if self.globals.contains_key(&name) {
            self.globals.insert(name, value);
            Ok(())
        } else {
            Err(Error::undefined_variable(&name))
        }
    }

    fn define_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    fn get_local(&self, slot: usize) -> Result<Value> {
        self.stack
            .get(self.call_stack.offset() + slot)
            .cloned()
            .ok_or(Error::stack_underflow())
    }

    fn set_local(&mut self, slot: usize, value: Value) -> Result<()> {
        let index = self.call_stack.offset() + slot;
        if index >= self.stack.len() {
            return Err(Error::stack_underflow());
        }
        self.stack[index] = value;
        Ok(())
    }

    /// Create an array from stack values
    fn create_array(&mut self, element_count: usize) -> Result<Value> {
        let mut elements = Vec::with_capacity(element_count);
        // Pop elements in reverse order (they were pushed in forward order)
        for _ in 0..element_count {
            elements.push(self.pop()?);
        }
        elements.reverse(); // Restore original order
        Ok(Value::Array(elements))
    }

    /// Index into an array
    fn index_array(&mut self) -> Result<()> {
        let index = self.pop()?;
        let array = self.pop()?;

        match (&array, &index) {
            (Value::Array(arr), Value::Number(idx)) => {
                let idx = *idx as usize;
                if idx >= arr.len() {
                    return Err(Error::runtime(format!("Array index {} out of bounds", idx)));
                }
                self.push(arr[idx].clone());
                Ok(())
            }
            (Value::Array(_), _) => Err(Error::runtime("Array index must be a number".to_string())),
            _ => Err(Error::runtime("Can only index arrays".to_string())),
        }
    }

    /// Set array element
    fn set_array_element(&mut self) -> Result<()> {
        let value = self.pop()?;
        let index = self.pop()?;
        let array = self.pop()?;

        match (&array, &index) {
            (Value::Array(arr), Value::Number(idx)) => {
                let idx = *idx as usize;
                if idx >= arr.len() {
                    return Err(Error::runtime(format!("Array index {} out of bounds", idx)));
                }
                // Create a new array with the modified element
                let mut new_arr = arr.clone();
                new_arr[idx] = value;
                self.push(Value::Array(new_arr));
                Ok(())
            }
            (Value::Array(_), _) => Err(Error::runtime("Array index must be a number".to_string())),
            _ => Err(Error::runtime("Can only index arrays".to_string())),
        }
    }

    fn print_values(&mut self, count: usize) -> Result<()> {
        for _ in 0..count {
            let value = self.pop()?;
            writeln!(self.output, "{value}")?;
        }
        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug utilities for the VM
impl VM {
    /// Print the current stack state
    pub fn debug_stack(&self) {
        todo!("Debug print stack")
    }

    /// Print instruction at current IP
    pub fn debug_instruction(&self) {
        todo!("Debug print current instruction")
    }

    /// Disassemble the current chunk
    pub fn disassemble_chunk(&self, name: &str) {
        if let Some(chunk) = &self.chunk {
            chunk.disassemble(name);
        }
    }
}
