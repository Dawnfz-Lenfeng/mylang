use super::{
    value::{Proto, Value},
    OpCode,
};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    globals: Vec<String>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            globals: Vec::new(),
        }
    }

    pub fn code(&self, ip: usize) -> u8 {
        self.code[ip]
    }

    pub fn constant(&self, index: usize) -> &Value {
        &self.constants[index]
    }

    pub fn global(&self, index: usize) -> &String {
        &self.globals[index]
    }

    pub fn current_ip(&self) -> usize {
        self.code.len()
    }
}

/// Write operations
impl Chunk {
    pub fn write(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        if let Some(index) = self.constants.iter().position(|v| v == &value) {
            return index as u8;
        }

        self.constants.push(value);
        self.constants.len() as u8 - 1
    }

    pub fn add_global(&mut self, name: String) -> u8 {
        if let Some(index) = self.resolve_global(&name) {
            return index;
        }

        self.globals.push(name);
        self.globals.len() as u8 - 1
    }

    pub fn resolve_global(&self, name: &str) -> Option<u8> {
        self.globals.iter().position(|s| s == name).map(|i| i as u8)
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_ip() - offset - 2; // 2 is the length of the jump instruction
        self.code[offset] = (jump >> 8) as u8;
        self.code[offset + 1] = jump as u8;
    }

    pub fn end_with_return(&mut self) {
        if let Some(op) = self.code.last() {
            if *op == OpCode::Return as u8 {
                return;
            }
        }
        self.write(OpCode::Nil as u8);
        self.write(OpCode::Return as u8);
    }
}

/// Debug utilities for the Chunk
impl Chunk {
    pub fn disassemble(&self, name: &str) {
        self.disassemble_recursive(name, 0);
    }

    fn disassemble_recursive(&self, name: &str, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{indent}== {name} ==");

        if !self.constants.is_empty() {
            println!("{indent}=== Constants ===");
            for (i, constant) in self.constants.iter().enumerate() {
                match constant {
                    Value::Proto(function) => {
                        let Proto {
                            name,
                            params,
                            start_ip,
                            ..
                        } = function.as_ref();
                        println!(
                            "{indent}constants[{i}] = function {name}({params_str}) at @{start_ip}",
                            params_str = params.join(", ")
                        );
                    }
                    _ => {
                        println!("{indent}constants[{i}] = {constant:?}");
                    }
                }
            }
        }

        if !self.globals.is_empty() {
            println!("{indent}=== Globals ===");
            for (i, global) in self.globals.iter().enumerate() {
                println!("{indent}globals[{i}] = {global:?}");
            }
        }

        println!("{indent}=== Code ===");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction_with_indent(offset, depth);
        }
    }

    fn disassemble_instruction_with_indent(&self, offset: usize, depth: usize) -> usize {
        let indent = "  ".repeat(depth);
        let instruction = self.code[offset];
        let op = OpCode::try_from(instruction).unwrap_or_else(|_| {
            println!("{indent}{offset:4} UNKNOWN_OP {instruction}");
            return OpCode::Nil; // fallback
        });

        match op {
            OpCode::Constant => {
                let operand = self.code[offset + 1];
                print!("{indent}{offset:4} {:15}", op);
                if let Some(constant) = self.constants.get(operand as usize) {
                    match constant {
                        Value::Proto(function) => {
                            let Proto { name, params, .. } = function.as_ref();
                            println!(" {} ; function {}({})", operand, name, params.join(", "));
                        }
                        _ => {
                            println!(" {} ; {:?}", operand, constant);
                        }
                    }
                } else {
                    println!(" {} ; INVALID_CONSTANT", operand);
                }
                offset + 2
            }
            OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::SetGlobal => {
                let operand = self.code[offset + 1];
                print!("{indent}{offset:4} {:15}", op);
                if let Some(global) = self.globals.get(operand as usize) {
                    println!(" {} ; {:?}", operand, global);
                } else {
                    println!(" {} ; INVALID_GLOBAL", operand);
                }
                offset + 2
            }
            OpCode::GetLocal | OpCode::SetLocal => {
                let operand = self.code[offset + 1];
                println!("{indent}{offset:4} {op:15} {operand} ; local[{operand}]");
                offset + 2
            }
            OpCode::JumpIfFalse | OpCode::Jump => {
                let high = self.code[offset + 1] as u16;
                let low = self.code[offset + 2] as u16;
                let jump_offset = (high << 8) | low;
                println!(
                    "{indent}{offset:4} {op:15} ; -> {}",
                    offset + 3 + jump_offset as usize
                );
                offset + 3
            }
            OpCode::Loop => {
                let operand = self.code[offset + 1];
                println!(
                    "{indent}{offset:4} {op:15} {operand} ; -> {}",
                    offset + 2 - operand as usize
                );
                offset + 2
            }
            OpCode::Call => {
                let arg_count = self.code[offset + 1] as usize;
                println!("{indent}{offset:4} {op:15} {arg_count} ; call");
                offset + 2
            }
            OpCode::Array => {
                let element_count = self.code[offset + 1] as usize;
                println!("{indent}{offset:4} {op:15} {element_count} ; create array with {element_count} elements");
                offset + 2
            }
            OpCode::Index => {
                println!("{indent}{offset:4} {op:15} ; array[index]");
                offset + 1
            }
            OpCode::IndexSet => {
                println!("{indent}{offset:4} {op:15} ; array[index] = value");
                offset + 1
            }
            OpCode::Closure => {
                let proto_index = self.code[offset + 1];
                let upvalue_count = self.code[offset + 2];
                print!("{indent}{offset:4} {op:15} {proto_index} ; ");

                if let Some(constant) = self.constants.get(proto_index as usize) {
                    if let Value::Proto(proto) = constant {
                        println!(
                            "closure for function '{}' with {} upvalues",
                            proto.name, upvalue_count
                        );

                        // Print upvalue details
                        let mut current_offset = offset + 3;
                        for i in 0..upvalue_count {
                            let is_local = self.code[current_offset];
                            let index = self.code[current_offset + 1];
                            println!(
                                "{indent}     upvalue[{}]: {} index {}",
                                i,
                                if is_local == 1 { "local" } else { "upvalue" },
                                index
                            );
                            current_offset += 2;
                        }

                        return current_offset;
                    }
                }
                println!("INVALID_PROTO");
                offset + 3 + (upvalue_count as usize * 2)
            }
            OpCode::GetUpvalue | OpCode::SetUpvalue => {
                let upvalue_index = self.code[offset + 1];
                println!("{indent}{offset:4} {op:15} {upvalue_index} ; upvalue[{upvalue_index}]");
                offset + 2
            }
            OpCode::Print => {
                let count = self.code[offset + 1] as usize;
                println!("{indent}{offset:4} {op:15} {count} ; print");
                offset + 2
            }
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
            | OpCode::Or => {
                println!("{indent}{offset:4} {op:15} ; binary operation");
                offset + 1
            }
            _ => {
                println!("{indent}{offset:4} {:?}", op);
                offset + 1
            }
        }
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chunk {{ code: {:?}, constants: {:?}, globals: {:?} }}",
            self.code, self.constants, self.globals
        )
    }
}
