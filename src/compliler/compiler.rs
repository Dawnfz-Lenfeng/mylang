use super::{
    chunk::Chunk,
    env::{Env, EnvRef},
    opcode::OpCode,
    value::{Proto, Value},
};
use crate::{
    error::{Error, Result},
    location::Location,
    parser::{expr, stmt, BinaryOp, Expr, LocatedStmt, Stmt, UnaryOp},
};

pub struct Compiler {
    chunk: Chunk,
    env: EnvRef,
    location: Location,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            env: Env::new_global(),
            location: Location::new(),
        }
    }

    pub fn compile(mut self, stmts: &[LocatedStmt]) -> Result<Chunk> {
        for stmt in stmts {
            self.location = stmt.location();
            stmt.as_inner()
                .accept(&mut self)
                .map_err(|e| e.at_location(self.location))?;
        }
        Ok(self.chunk)
    }
}

impl Compiler {
    fn begin_scope(&mut self) {
        self.env.borrow_mut().begin_scope();
    }

    fn end_scope(&mut self) -> Result<()> {
        let pop_count = self.env.borrow_mut().end_scope()?;
        for _ in 0..pop_count {
            self.emit_op(OpCode::Pop);
        }

        Ok(())
    }

    fn begin_loop(&mut self) {
        self.env.borrow_mut().begin_loop();
    }

    fn end_loop(&mut self, continue_target: usize) -> Result<()> {
        if let Some(loop_context) = self.env.borrow_mut().end_loop() {
            for break_jump in loop_context.break_jumps {
                self.chunk.patch_jump(break_jump);
            }
            for continue_jump in loop_context.continue_jumps {
                self.chunk
                    .patch_jump_with_target(continue_jump, continue_target);
            }
        }

        Ok(())
    }

    fn begin_enclosed_scope(&mut self) {
        let enclosed = Env::new_enclosed(self.env.clone());
        self.env = enclosed;
    }

    fn end_enclosed_scope(&mut self) -> Result<()> {
        if self.env.borrow().is_global() {
            return Err(Error::quit_from_global());
        }

        let num_locals = self.env.borrow().locals.len();
        for _ in 0..num_locals {
            self.emit_op(OpCode::Pop);
        }

        let enclosing = self.env.borrow_mut().enclosing.take().unwrap();
        self.env = enclosing;

        Ok(())
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.chunk.add_constant(value);
        self.emit_op_with_operand(OpCode::Constant, index);
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit_byte(op as u8);
        let offset = self.chunk.current_ip();
        self.emit_byte(0);
        self.emit_byte(0);
        offset
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write_with_location(byte, self.location);
    }

    fn emit_op(&mut self, op: OpCode) {
        self.emit_byte(op as u8);
    }

    fn emit_op_with_operand(&mut self, op: OpCode, operand: u8) {
        self.emit_byte(op as u8);
        self.emit_byte(operand);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.chunk.current_ip() - loop_start + 3; // +3 for the jump instruction
        self.emit_byte(OpCode::Loop as u8);
        self.emit_byte((offset >> 8) as u8);
        self.emit_byte(offset as u8);
    }
}

impl stmt::Visitor<Result<()>> for Compiler {
    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        expr.accept(self)?;
        self.emit_op(OpCode::Pop); // Pop the result of expression statement
        Ok(())
    }

    fn visit_print(&mut self, exprs: &[Expr]) -> Result<()> {
        for expr in exprs {
            expr.accept(self)?;
        }
        self.emit_op_with_operand(OpCode::Print, exprs.len() as u8);
        Ok(())
    }

    fn visit_var_decl(&mut self, name: &str, initializer: Option<&Expr>) -> Result<()> {
        if let Some(initializer) = initializer {
            initializer.accept(self)?;
        } else {
            self.emit_op(OpCode::Nil);
        }

        if self.env.borrow().is_global() {
            let global_index = self.chunk.add_global(name.to_string());
            self.emit_op_with_operand(OpCode::DefineGlobal, global_index as u8);
        } else {
            self.env.borrow_mut().add_local(name.to_string());
        }
        Ok(())
    }

    fn visit_func_decl(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<()> {
        // predeclare function name for recursion support
        let index = if self.env.borrow().is_global() {
            Some(self.chunk.add_global(name.to_string()))
        } else {
            self.env.borrow_mut().add_local(name.to_string());
            None
        };

        let skip = self.emit_jump(OpCode::Jump); // jump to create function with upvalues
        let start_ip = self.chunk.current_ip();

        self.begin_enclosed_scope(); // new enclosed env

        self.env.borrow_mut().add_locals(params);
        for stmt in body {
            stmt.accept(self)?;
        }
        self.chunk.end_with_return();
        let upvalues = self.env.borrow().upvalues.clone();

        self.end_enclosed_scope()?;

        self.chunk.patch_jump(skip); // jump here

        let proto = Value::Proto(Proto {
            name: name.to_string(),
            params: params.to_vec(),
            start_ip,
            upvalues: upvalues.clone(),
        });
        let proto_index = self.chunk.add_constant(proto);

        self.emit_op_with_operand(OpCode::Closure, proto_index);
        self.emit_byte(upvalues.len() as u8);
        for upvalue in &upvalues {
            self.emit_byte(if upvalue.is_local { 1 } else { 0 });
            self.emit_byte(upvalue.index as u8);
        }

        if let Some(index) = index {
            self.emit_op_with_operand(OpCode::DefineGlobal, index);
        }

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
        self.begin_loop();

        let loop_start = self.chunk.current_ip();

        condition.accept(self)?;
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

        body.accept(self)?;
        self.emit_loop(loop_start);

        self.chunk.patch_jump(exit_jump);

        self.end_loop(loop_start)?;

        Ok(())
    }

    fn visit_for(
        &mut self,
        initializer: Option<&Stmt>,
        condition: &Expr,
        increment: Option<&Expr>,
        body: &Stmt,
    ) -> Result<()> {
        self.begin_scope();

        if let Some(init) = initializer {
            init.accept(self)?;
        }

        let loop_start = self.chunk.current_ip();

        self.begin_loop();

        condition.accept(self)?;
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

        body.accept(self)?;
        let continue_target = if let Some(inc) = increment {
            let target = self.chunk.current_ip();
            inc.accept(self)?;
            self.emit_op(OpCode::Pop); // pop the increment value
            target
        } else {
            loop_start
        };
        self.emit_loop(loop_start);

        self.chunk.patch_jump(exit_jump);

        self.end_loop(continue_target)?;

        self.end_scope()?;

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
        if !self.env.borrow().in_loop() {
            return Err(Error::compilation("break outside of loop".to_string()));
        }

        // pop locals that are in the loop
        let pop_count = self.env.borrow().get_loop_locals_to_pop();
        for _ in 0..pop_count {
            self.emit_op(OpCode::Pop);
        }

        let jump_position = self.emit_jump(OpCode::Jump);
        self.env.borrow_mut().add_break_jump(jump_position)?;
        Ok(())
    }

    fn visit_continue(&mut self) -> Result<()> {
        if !self.env.borrow().in_loop() {
            return Err(Error::compilation("continue outside of loop".to_string()));
        }

        // pop locals that are in the loop
        let pop_count = self.env.borrow().get_loop_locals_to_pop();
        for _ in 0..pop_count {
            self.emit_op(OpCode::Pop);
        }

        let jump_position = self.emit_jump(OpCode::Jump);
        self.env.borrow_mut().add_continue_jump(jump_position)?;
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

impl expr::Visitor<Result<()>> for Compiler {
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
        let (op, index) = {
            let mut env = self.env.borrow_mut();
            if let Some(local_index) = env.resolve_local(name) {
                (OpCode::GetLocal, local_index)
            } else if let Some(upvalue_index) = env.resolve_upvalue(name) {
                (OpCode::GetUpvalue, upvalue_index)
            } else if let Some(global_index) = self.chunk.resolve_global(name) {
                (OpCode::GetGlobal, global_index)
            } else {
                return Err(Error::undefined_variable(name));
            }
        };

        self.emit_op_with_operand(op, index);
        Ok(())
    }

    fn visit_array(&mut self, elements: &[Expr]) -> Result<()> {
        for element in elements {
            element.accept(self)?;
        }
        self.emit_op_with_operand(OpCode::Array, elements.len() as u8);
        Ok(())
    }

    fn visit_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<()> {
        match op {
            BinaryOp::LogicalAnd => {
                left.accept(self)?;
                self.emit_op(OpCode::Dup); // keep left value on stack
                let left_jump = self.emit_jump(OpCode::JumpIfFalse);

                self.emit_op(OpCode::Pop); // pop left value
                right.accept(self)?;
                let right_jump = self.emit_jump(OpCode::Jump);

                self.chunk.patch_jump(left_jump);
                self.chunk.patch_jump(right_jump);
            }
            BinaryOp::LogicalOr => {
                left.accept(self)?;
                self.emit_op(OpCode::Dup);
                let left_jump = self.emit_jump(OpCode::JumpIfTrue);

                self.emit_op(OpCode::Pop);
                right.accept(self)?;
                let right_jump = self.emit_jump(OpCode::Jump);

                self.chunk.patch_jump(left_jump);
                self.chunk.patch_jump(right_jump);
            }
            _ => {
                // 其他二进制操作正常处理
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
                    _ => unreachable!(),
                }
            }
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

        let (op, index) = {
            let mut env = self.env.borrow_mut();
            if let Some(local_index) = env.resolve_local(name) {
                (OpCode::SetLocal, local_index)
            } else if let Some(upvalue_index) = env.resolve_upvalue(name) {
                (OpCode::SetUpvalue, upvalue_index)
            } else if let Some(global_index) = self.chunk.resolve_global(name) {
                (OpCode::SetGlobal, global_index)
            } else {
                return Err(Error::compilation(format!(
                    "Assignment to undeclared variable '{name}'"
                )));
            }
        };

        self.emit_op_with_operand(op, index);
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
        for argument in arguments {
            argument.accept(self)?;
        }
        callee.accept(self)?;
        self.emit_op_with_operand(OpCode::Call, arguments.len() as u8);
        Ok(())
    }
}
