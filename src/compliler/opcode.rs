use crate::error::{Error, Result};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Constants
    Constant = 0,
    Nil,
    True,
    False,

    // Arithmetic
    Add = 10,
    Subtract,
    Multiply,
    Divide,
    Negate,

    // Comparison
    Equal = 20,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

    // Logical
    Not = 30,
    And,
    Or,

    // Variables
    DefineGlobal = 40,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,

    // Control flow
    Jump = 50,
    JumpIfFalse,
    Loop,
    Break,
    Continue,

    // Functions
    Call = 60,
    Return,

    // Stack operations
    Pop = 70,
    Print,

    // Arrays
    Array = 80,
    Index,
    IndexSet,

    // Closures and Upvalues
    Closure = 90, // Create closure from function prototype
    GetUpvalue,   // Get upvalue value
    SetUpvalue,   // Set upvalue value
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        op as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self> {
        match byte {
            0 => Ok(OpCode::Constant),
            1 => Ok(OpCode::Nil),
            2 => Ok(OpCode::True),
            3 => Ok(OpCode::False),
            10 => Ok(OpCode::Add),
            11 => Ok(OpCode::Subtract),
            12 => Ok(OpCode::Multiply),
            13 => Ok(OpCode::Divide),
            14 => Ok(OpCode::Negate),
            20 => Ok(OpCode::Equal),
            21 => Ok(OpCode::NotEqual),
            22 => Ok(OpCode::LessThan),
            23 => Ok(OpCode::LessEqual),
            24 => Ok(OpCode::GreaterThan),
            25 => Ok(OpCode::GreaterEqual),
            30 => Ok(OpCode::Not),
            31 => Ok(OpCode::And),
            32 => Ok(OpCode::Or),
            40 => Ok(OpCode::DefineGlobal),
            41 => Ok(OpCode::GetGlobal),
            42 => Ok(OpCode::SetGlobal),
            43 => Ok(OpCode::GetLocal),
            44 => Ok(OpCode::SetLocal),
            50 => Ok(OpCode::Jump),
            51 => Ok(OpCode::JumpIfFalse),
            52 => Ok(OpCode::Loop),
            53 => Ok(OpCode::Break),
            54 => Ok(OpCode::Continue),
            60 => Ok(OpCode::Call),
            61 => Ok(OpCode::Return),
            70 => Ok(OpCode::Pop),
            71 => Ok(OpCode::Print),
            80 => Ok(OpCode::Array),
            81 => Ok(OpCode::Index),
            82 => Ok(OpCode::IndexSet),
            90 => Ok(OpCode::Closure),
            91 => Ok(OpCode::GetUpvalue),
            92 => Ok(OpCode::SetUpvalue),
            _ => Err(Error::invalid_opcode(byte)),
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
