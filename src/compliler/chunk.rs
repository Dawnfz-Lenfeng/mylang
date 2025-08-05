use super::{value::Value, OpCode};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub globals: Vec<String>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            globals: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_op(&mut self, op: OpCode, arg: u8, line: usize) {
        self.write(op as u8, line);
        self.write(arg, line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        if let Some(index) = self.constants.iter().position(|v| v == &value) {
            return index;
        }

        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn add_global(&mut self, global: String) -> usize {
        if let Some(index) = self.globals.iter().position(|s| s == &global) {
            return index;
        }

        self.globals.push(global);
        self.globals.len() - 1
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump = self.code.len() - offset - 2;
        self.code[offset] = (jump >> 8) as u8;
        self.code[offset + 1] = jump as u8;
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        let instruction = self.code[offset];
        let op = OpCode::try_from(instruction).unwrap_or_else(|_| {
            println!("{:4} UNKNOWN_OP {}", offset, instruction);
            return OpCode::Nil; // fallback
        });
        
        match op {
            OpCode::Constant => {
                let operand = self.code[offset + 1];
                print!("{:4} {:15}", offset, format!("{:?}", op));
                if let Some(constant) = self.constants.get(operand as usize) {
                    println!(" {} ; {:?}", operand, constant);
                } else {
                    println!(" {} ; INVALID_CONSTANT", operand);
                }
                offset + 2
            }
            OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::SetGlobal => {
                let operand = self.code[offset + 1];
                print!("{:4} {:15}", offset, format!("{:?}", op));
                if let Some(global) = self.globals.get(operand as usize) {
                    println!(" {} ; {:?}", operand, global);
                } else {
                    println!(" {} ; INVALID_GLOBAL", operand);
                }
                offset + 2
            }
            OpCode::GetLocal | OpCode::SetLocal => {
                let operand = self.code[offset + 1];
                println!("{:4} {:15} {} ; local[{}]", offset, format!("{:?}", op), operand, operand);
                offset + 2
            }
            OpCode::JumpIfFalse | OpCode::Jump => {
                let high = self.code[offset + 1] as u16;
                let low = self.code[offset + 2] as u16;
                let jump_offset = (high << 8) | low;
                println!("{:4} {:15} {} ; -> {}", offset, format!("{:?}", op), jump_offset, offset + 3 + jump_offset as usize);
                offset + 3
            }            
            OpCode::Loop => {
                let operand = self.code[offset + 1];
                println!("{:4} {:15} {} ; -> {}", offset, format!("{:?}", op), operand, offset + 2 - operand as usize);
                offset + 2
            }
            _ => {
                println!("{:4} {:?}", offset, op);
                offset + 1
            }
        }
    }
}
