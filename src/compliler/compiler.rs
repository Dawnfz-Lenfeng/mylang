use super::{chunk::Chunk, opcode::OpCode, value::Value};
use crate::{
    error::{Error, Result},
    parser::{
        expr::{self, BinaryOp, Expr, UnaryOp},
        stmt::{self, Stmt},
    },
};

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: usize,
    pub is_captured: bool,
}

pub struct Compiler<'a> {
    chunk: &'a mut Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    current_line: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        Self {
            chunk,
            locals: Vec::new(),
            scope_depth: 0,
            current_line: 0,
        }
    }

    pub fn compile(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            stmt.accept(self)?;
        }
        Ok(())
    }

    fn emit_constant(&mut self, value: Value) -> usize {
        let index = self.chunk.add_constant(value);
        self.emit_op_with_operand(OpCode::Constant, index as u8);
        index
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) -> Result<()> {
        if self.scope_depth == 0 {
            return Err(Error::runtime("Cannot end scope with depth 0".to_string()));
        }

        self.scope_depth -= 1;

        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.locals.pop();
                self.emit_op(OpCode::Pop);
            } else {
                break;
            }
        }
        Ok(())
    }

    fn add_global(&mut self, global: String) -> usize {
        self.chunk.add_global(global)
    }

    fn add_local(&mut self, name: String) -> usize {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
            is_captured: false,
        });
        self.locals.len() - 1
    }

    fn resolve_local(&mut self, name: &str) -> Option<u8> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find(|(_, local)| local.name == name)
            .map(|(index, _)| index as u8)
    }

    fn resolve_global(&mut self, name: &str) -> Option<u8> {
        self.chunk
            .globals
            .iter()
            .enumerate()
            .rev()
            .find(|(_, global)| global == &name)
            .map(|(index, _)| index as u8)
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        let offset = self.chunk.code.len();
        self.emit_byte(op as u8);
        self.emit_byte(0);
        self.emit_byte(0);
        offset
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte, self.current_line);
    }

    fn emit_op(&mut self, op: OpCode) {
        self.emit_byte(op as u8);
    }

    fn emit_op_with_operand(&mut self, op: OpCode, operand: u8) {
        self.emit_byte(op as u8);
        self.emit_byte(operand);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.chunk.code.len() - loop_start;
        self.emit_byte(OpCode::Loop as u8);
        self.emit_byte(offset as u8);
    }
}

impl<'a> stmt::Visitor<Result<()>> for Compiler<'a> {
    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_print(&mut self, exprs: &[Expr]) -> Result<()> {
        for expr in exprs {
            expr.accept(self)?;
        }
        Ok(())
    }

    fn visit_var_decl(&mut self, name: &str, initializer: Option<&Expr>) -> Result<()> {
        if let Some(initializer) = initializer {
            initializer.accept(self)?;
        } else {
            self.emit_op(OpCode::Nil);
        }

        if self.scope_depth == 0 {
            let global_index = self.add_global(name.to_string());
            self.emit_op_with_operand(OpCode::DefineGlobal, global_index as u8);
        } else {
            self.add_local(name.to_string());
        }
        Ok(())
    }

    fn visit_func_decl(&mut self, name: &str, params: &[String], body: &Stmt) -> Result<()> {
        let func_value = Value::Function {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.clone(),
        };
        let func_index = self.chunk.add_constant(func_value);
        let name_index = self.add_global(name.to_string());
        self.emit_op_with_operand(OpCode::Constant, func_index as u8);
        self.emit_op_with_operand(OpCode::DefineGlobal, name_index as u8);
        
        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<()> {
        condition.accept(self)?;

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        then_branch.accept(self)?;

        let else_jump = self.emit_jump(OpCode::Jump);

        self.chunk.patch_jump(then_jump);

        if let Some(else_branch) = else_branch {
            else_branch.accept(self)?;
        }

        self.chunk.patch_jump(else_jump);
        Ok(())
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        let loop_start = self.chunk.code.len();
        condition.accept(self)?;
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        body.accept(self)?;
        self.emit_loop(loop_start);
        self.chunk.patch_jump(exit_jump);
        Ok(())
    }

    fn visit_return(&mut self, value: Option<&Expr>) -> Result<()> {
        if let Some(value) = value {
            value.accept(self)?;
        } else {
            self.emit_op(OpCode::Nil);
        }
        self.emit_op(OpCode::Return);
        Ok(())
    }

    fn visit_break(&mut self) -> Result<()> {
        self.emit_op(OpCode::Break);
        Ok(())
    }

    fn visit_continue(&mut self) -> Result<()> {
        self.emit_op(OpCode::Continue);
        Ok(())
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Result<()> {
        self.begin_scope();
        for stmt in statements {
            stmt.accept(self)?;
        }
        self.end_scope()?;
        Ok(())
    }
}

impl<'a> expr::Visitor<Result<()>> for Compiler<'a> {
    fn visit_number(&mut self, value: f64) -> Result<()> {
        self.emit_constant(Value::Number(value));
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Result<()> {
        self.emit_constant(Value::String(value.to_string()));
        Ok(())
    }

    fn visit_boolean(&mut self, value: bool) -> Result<()> {
        self.emit_constant(Value::Boolean(value));
        Ok(())
    }

    fn visit_nil(&mut self) -> Result<()> {
        self.emit_op(OpCode::Nil);
        Ok(())
    }

    fn visit_identifier(&mut self, name: &str) -> Result<()> {
        if let Some(local_index) = self.resolve_local(name) {
            self.emit_op_with_operand(OpCode::GetLocal, local_index);
        } else if let Some(global_index) = self.resolve_global(name) {
            self.emit_op_with_operand(OpCode::GetGlobal, global_index);
        } else {
            return Err(Error::runtime(format!("Undefined variable '{name}'")));
        }
        Ok(())
    }

    fn visit_array(&mut self, elements: &[Expr]) -> Result<()> {
        self.emit_op(OpCode::Array);
        for element in elements {
            element.accept(self)?;
        }
        Ok(())
    }

    fn visit_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<()> {
        left.accept(self)?;
        right.accept(self)?;
        match op {
            BinaryOp::Add => self.emit_op(OpCode::Add),
            BinaryOp::Subtract => self.emit_op(OpCode::Subtract),
            BinaryOp::Multiply => self.emit_op(OpCode::Multiply),
            BinaryOp::Divide => self.emit_op(OpCode::Divide),
            BinaryOp::Equal => self.emit_op(OpCode::Equal),
            BinaryOp::NotEqual => self.emit_op(OpCode::NotEqual),
            BinaryOp::LessThan => self.emit_op(OpCode::LessThan),
            BinaryOp::LessEqual => self.emit_op(OpCode::LessEqual),
            BinaryOp::GreaterThan => self.emit_op(OpCode::GreaterThan),
            BinaryOp::GreaterEqual => self.emit_op(OpCode::GreaterEqual),
            BinaryOp::LogicalAnd => self.emit_op(OpCode::And),
            BinaryOp::LogicalOr => self.emit_op(OpCode::Or),
        }
        Ok(())
    }

    fn visit_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<()> {
        operand.accept(self)?;
        match op {
            UnaryOp::Negate => self.emit_op(OpCode::Negate),
            UnaryOp::Not => self.emit_op(OpCode::Not),
        }
        Ok(())
    }

    fn visit_assign(&mut self, name: &str, value: &Expr) -> Result<()> {
        value.accept(self)?;
        if let Some(local_index) = self.resolve_local(name) {
            self.emit_op_with_operand(OpCode::SetLocal, local_index);
        } else if let Some(global_index) = self.resolve_global(name) {
            self.emit_op_with_operand(OpCode::SetGlobal, global_index);
        } else {
            return Err(Error::runtime(format!("Assignment to undeclared variable '{name}'")));
        }
        Ok(())
    }

    fn visit_index_assign(&mut self, array: &Expr, index: &Expr, value: &Expr) -> Result<()> {
        array.accept(self)?;
        index.accept(self)?;
        value.accept(self)?;
        self.emit_op(OpCode::IndexSet);
        Ok(())
    }

    fn visit_index(&mut self, array: &Expr, index: &Expr) -> Result<()> {
        array.accept(self)?;
        index.accept(self)?;
        self.emit_op(OpCode::Index);
        Ok(())
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> Result<()> {
        callee.accept(self)?;
        for argument in arguments {
            argument.accept(self)?;
        }
        self.emit_op(OpCode::Call);
        Ok(())
    }
}
