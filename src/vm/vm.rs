use super::stack::{CallFrame, CallStack};
use crate::{
    compliler::{Chunk, Function, OpCode, Value, BUILTIN_FUNCTIONS},
    constant::STACK_SIZE,
    error::{Error, Result},
};
use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    call_stack: CallStack,
    output: Box<dyn Write>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        let globals = BUILTIN_FUNCTIONS
            .iter()
            .map(|(name, func)| {
                (
                    name.to_string(),
                    Value::BuiltinFunction {
                        name: name.to_string(),
                        function: *func,
                    },
                )
            })
            .collect();
        Self {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE),
            globals,
            call_stack: CallStack::new(),
            output: Box::new(std::io::stdout()),
        }
    }

    pub fn with_output(chunk: Chunk, output: Box<dyn Write>) -> Self {
        Self {
            output,
            ..Self::new(chunk)
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if self.ip >= self.chunk.current_ip() {
                break;
            }

            let location = self.chunk.location_at(self.ip);
            let instruction = OpCode::try_from(self.read_byte()?)?;
            self.run_instruction(instruction)
                .map_err(|e| e.at_location(location))?;
        }
        Ok(())
    }
}

/// Utility functions
impl VM {
    fn read_byte(&mut self) -> Result<u8> {
        let byte = self
            .chunk
            .code(self.ip)
            .ok_or(Error::code_out_of_bounds(self.ip, self.chunk.current_ip()))?;
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
        Ok(self
            .chunk
            .constant(index as usize)
            .ok_or(Error::constant_overflow())?
            .clone())
    }

    fn read_global_name(&mut self) -> Result<String> {
        let index = self.read_byte()?;
        Ok(self
            .chunk
            .global(index as usize)
            .ok_or(Error::global_overflow())?
            .clone())
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::stack_underflow())
    }

    fn peek(&self) -> Result<Value> {
        self.stack.last().cloned().ok_or(Error::stack_underflow())
    }
}

/// Instruction execution
impl VM {
    fn run_instruction(&mut self, instruction: OpCode) -> Result<()> {
        match instruction {
            OpCode::Constant => {
                let constant = self.read_constant()?;
                self.push(constant);
            }
            OpCode::Nil => self.push(Value::Nil),
            OpCode::True => self.push(Value::Boolean(true)),
            OpCode::False => self.push(Value::Boolean(false)),

            OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Equal
            | OpCode::NotEqual
            | OpCode::LessThan
            | OpCode::LessEqual
            | OpCode::GreaterThan
            | OpCode::GreaterEqual
            | OpCode::And
            | OpCode::Or => self.binary_op(instruction)?,

            OpCode::Negate | OpCode::Not => self.unary_op(instruction)?,

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
                let value = self.peek()?;
                self.set_global(name, value)?;
            }
            OpCode::GetLocal => {
                let slot = self.read_byte()? as usize;
                let value = self.get_local(slot)?;
                self.push(value);
            }
            OpCode::SetLocal => {
                let slot = self.read_byte()? as usize;
                let value = self.peek()?;
                self.set_local(slot, value)?;
            }

            // Control flow
            OpCode::Jump => {
                let offset = self.read_short()? as usize;
                self.ip += offset;
            }
            OpCode::JumpIfFalse => {
                let offset = self.read_short()? as usize;
                let condition = self.pop()?;
                if !condition.is_truthy() {
                    self.ip += offset;
                }
            }
            OpCode::Loop => {
                let offset = self.read_short()? as usize;
                self.ip -= offset;
            }

            // Functions
            OpCode::Call => {
                let arg_count = self.read_byte()? as usize;
                let callee = self.pop()?;
                self.call_value(callee, arg_count)?;
            }
            OpCode::Return => {
                let result = self.pop()?;
                if let Some(frame) = self.call_stack.pop() {
                    self.ip = frame.ip;
                    self.stack.truncate(frame.slots_offset);
                    self.push(result);
                } else {
                    return Ok(());
                }
            }

            // Stack operations
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::Print => {
                let count = self.read_byte()? as usize;
                self.print_values(count)?;
            }

            // Arrays
            OpCode::Array => {
                let element_count = self.read_byte()? as usize;
                let array = self.create_array(element_count)?;
                self.push(array);
            }
            OpCode::Index => {
                let index = self.pop()?;
                let array = self.pop()?;
                self.index_array(index, array)?;
            }
            OpCode::IndexSet => {
                let value = self.pop()?;
                let index = self.pop()?;
                let array = self.pop()?;
                self.set_array_element(value, index, array)?;
            }

            // Closures and Upvalues
            OpCode::Closure => {
                let proto = self.read_constant()?;
                let upvalue_count = self.read_byte()? as usize;
                self.create_closure(proto, upvalue_count)?;
            }
            OpCode::GetUpvalue => {
                let upvalue_index = self.read_byte()? as usize;
                self.get_upvalue(upvalue_index)?;
            }
            OpCode::SetUpvalue => {
                let upvalue_index = self.read_byte()? as usize;
                let value = self.peek()?;
                self.set_upvalue(upvalue_index, value)?;
            }
        }
        Ok(())
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
            Value::Function(function) => {
                if function.arity() != arg_count {
                    return Err(Error::arity_error(
                        &function.name,
                        function.arity(),
                        arg_count,
                    ));
                }

                let frame = CallFrame {
                    function: function.clone(),
                    ip: self.ip,
                    slots_offset: self.stack.len() - arg_count,
                };
                self.call_stack.push(frame);
                self.ip = function.start_ip;
                Ok(())
            }
            Value::BuiltinFunction { function, .. } => {
                let args: Vec<Value> = (0..arg_count)
                    .map(|_| self.pop())
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .rev()
                    .collect();

                let result = function(&args)?;
                self.push(result);
                Ok(())
            }
            _ => Err(Error::runtime(
                "Can only call functions and closures".to_string(),
            )),
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

    fn create_array(&mut self, element_count: usize) -> Result<Value> {
        let elements = (0..element_count)
            .map(|_| self.pop())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .rev()
            .collect();

        Ok(Value::Array(Rc::new(RefCell::new(elements))))
    }

    fn index_array(&mut self, index: Value, array: Value) -> Result<()> {
        match (&array, &index) {
            (Value::Array(arr), Value::Number(idx)) => {
                let idx = *idx as usize;
                let value = arr
                    .borrow()
                    .get(idx)
                    .ok_or(Error::index_out_of_bounds(idx, arr.borrow().len()))?
                    .clone();
                self.push(value);
                Ok(())
            }
            (Value::Array(_), _) => Err(Error::runtime("array index must be a number".to_string())),
            _ => Err(Error::runtime("can only index arrays".to_string())),
        }
    }

    fn set_array_element(&mut self, value: Value, index: Value, array: Value) -> Result<()> {
        match (&array, &index) {
            (Value::Array(arr), Value::Number(idx)) => {
                let idx = *idx as usize;
                if let Some(target) = arr.borrow_mut().get_mut(idx) {
                    *target = value.clone();
                    self.push(value);
                } else {
                    return Err(Error::index_out_of_bounds(idx, arr.borrow().len()));
                }
                Ok(())
            }
            (Value::Array(_), _) => Err(Error::runtime("array index must be a number".to_string())),
            _ => Err(Error::runtime("can only index arrays".to_string())),
        }
    }

    fn print_values(&mut self, count: usize) -> Result<()> {
        let output = (0..count)
            .map(|_| self.pop())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .rev()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        writeln!(self.output, "{output}")?;
        Ok(())
    }

    fn create_closure(&mut self, proto: Value, upvalue_count: usize) -> Result<()> {
        let proto = match proto {
            Value::Proto(proto) => proto,
            _ => {
                return Err(Error::runtime(
                    "Expected function in closure creation".to_string(),
                ))
            }
        };

        let upvalues = (0..upvalue_count)
            .map(|_| {
                let is_local = self.read_byte()? == 1;
                let index = self.read_byte()? as usize;
                if is_local {
                    let value = self.get_local(index)?;
                    Ok(Value::new_upvalue(value.clone()))
                } else {
                    self.call_stack.get_upvalue(index).cloned()
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let function = Value::Function(Rc::new(Function::from_proto(proto, upvalues)));
        self.push(function);

        Ok(())
    }

    fn get_upvalue(&mut self, upvalue_index: usize) -> Result<()> {
        let upvalue = self.call_stack.get_upvalue(upvalue_index)?;
        let value = upvalue.borrow().clone();
        self.push(value);
        Ok(())
    }

    fn set_upvalue(&mut self, upvalue_index: usize, value: Value) -> Result<()> {
        let upvalue = self.call_stack.get_upvalue(upvalue_index)?;
        *upvalue.borrow_mut() = value;
        Ok(())
    }
}
