use crate::ast::{DataType, Expr, Parameter, Program, Stmt, UnaryOp};
use crate::error::CompilerError;
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, CompilerError> {
        let mut program = Program::new();

        while !self.is_at_end() {
            if let Ok(stmt) = self.parse_statement() {
                program.add_statement(stmt);
            }
        }

        Ok(program)
    }

    /// Grammar: var_declaration | function_declaration | if_statement | while_statement | for_statement | return_statement | block_statement | expression_statement
    fn parse_statement(&mut self) -> Result<Stmt, CompilerError> {
        match self.peek().token_type {
            TokenType::Let => self.parse_var_declaration(),
            TokenType::Fn => self.parse_function_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::LeftBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    /// Grammar: 'let' IDENTIFIER [':' TYPE] ['=' EXPRESSION] ';'
    fn parse_var_declaration(&mut self) -> Result<Stmt, CompilerError> {
        self.advance(); // consume 'let'
        let name = self.consume_identifier()?;
        let type_annotation = self
            .check(&TokenType::Colon)
            .then(|| {
                self.advance();
                self.parse_type()
            })
            .transpose()?;
        let initializer = self
            .check(&TokenType::Assign)
            .then(|| {
                self.advance();
                self.parse_expression()
            })
            .transpose()?;
        self.consume_semicolon()?;
        Ok(Stmt::VarDecl {
            name,
            type_annotation,
            initializer,
        })
    }

    /// Grammar: 'fn' IDENTIFIER '(' [IDENTIFIER ':' TYPE] ')' '->' TYPE '{' STATEMENTS '}'
    fn parse_function_declaration(&mut self) -> Result<Stmt, CompilerError> {
        self.advance(); // consume 'fn'

        let name = self.consume_identifier()?;

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let parameters = self.parse_parameters()?;
        self.consume(
            TokenType::RightParen,
            "Expected ')' after function parameters",
        )?;

        let return_type = self
            .check(&TokenType::Arrow)
            .then(|| {
                self.advance();
                self.parse_type()
            })
            .transpose()?;

        let body = self.parse_block_statement_inner()?;

        Ok(Stmt::FuncDecl {
            name,
            parameters,
            return_type,
            body,
        })
    }

    /// Grammar: 'if' EXPRESSION '{' STATEMENTS '}' ['else' '{' STATEMENTS '}']
    fn parse_if_statement(&mut self) -> Result<Stmt, CompilerError> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block_statement_inner()?;
        let else_branch = self
            .check(&TokenType::Else)
            .then(|| {
                self.advance();
                self.parse_block_statement_inner()
            })
            .transpose()?;
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    /// Grammar: 'while' EXPRESSION '{' STATEMENTS '}'
    fn parse_while_statement(&mut self) -> Result<Stmt, CompilerError> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        let body = self.parse_block_statement_inner()?;
        Ok(Stmt::While { condition, body })
    }

    /// Grammar: 'for' IDENTIFIER 'in' EXPRESSION '{' STATEMENTS '}'
    fn parse_for_statement(&mut self) -> Result<Stmt, CompilerError> {
        self.advance(); // consume 'for'
        let name = self.consume_identifier()?;
        self.consume(TokenType::In, "Expected 'in' after for loop variable")?;
        let collection = self.parse_expression()?;
        let body = self.parse_block_statement_inner()?;
        Ok(Stmt::For {
            name,
            collection,
            body,
        })
    }

    /// Grammar: 'return' [EXPRESSION] ';'
    fn parse_return_statement(&mut self) -> Result<Stmt, CompilerError> {
        self.advance();
        let value = self
            .check(&TokenType::Semicolon)
            .then(|| self.parse_expression())
            .transpose()?;
        Ok(Stmt::Return { value })
    }

    /// Grammar: '{' STATEMENTS '}'
    fn parse_block_statement(&mut self) -> Result<Stmt, CompilerError> {
        let statements = self.parse_block_statement_inner()?;
        Ok(Stmt::Block(statements))
    }

    /// Grammar: EXPRESSION ';'
    fn parse_expression_statement(&mut self) -> Result<Stmt, CompilerError> {
        let expr = self.parse_expression()?;
        self.consume_semicolon()?;
        Ok(Stmt::Expression(expr))
    }

    fn parse_block_statement_inner(&mut self) -> Result<Vec<Stmt>, CompilerError> {
        self.consume(TokenType::LeftBrace, "Expected '{' at start of block")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            statements.push(self.parse_statement()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' at end of block")?;
        Ok(statements)
    }

    /// Grammar: assignment
    fn parse_expression(&mut self) -> Result<Expr, CompilerError> {
        self.parse_assignment()
    }

    /// Grammar: identifier '=' expression | or
    fn parse_assignment(&mut self) -> Result<Expr, CompilerError> {
        // TODO: Implement assignment parsing
        self.parse_or()
    }

    /// Grammar: and ('||' and)*
    fn parse_or(&mut self) -> Result<Expr, CompilerError> {
        // TODO: Implement logical OR parsing
        self.parse_and()
    }

    /// Grammar: equality ('&&' equality)*
    fn parse_and(&mut self) -> Result<Expr, CompilerError> {
        // TODO: Implement logical AND parsing
        self.parse_equality()
    }

    /// Grammar: comparison ('==' comparison | '!=' comparison)*
    fn parse_equality(&mut self) -> Result<Expr, CompilerError> {
        // TODO: Implement equality parsing
        self.parse_comparison()
    }

    /// Grammar: term ('<' term | '>' term | '<=' term | '>=' term)*
    fn parse_comparison(&mut self) -> Result<Expr, CompilerError> {
        // TODO: Implement comparison parsing
        self.parse_term()
    }

    /// Grammar: factor ('+' factor | '-' factor)*
    fn parse_term(&mut self) -> Result<Expr, CompilerError> {
        // TODO: Implement addition/subtraction parsing
        self.parse_factor()
    }

    /// Grammar: unary ('*' unary | '/' unary)*
    fn parse_factor(&mut self) -> Result<Expr, CompilerError> {
        // TODO: Implement multiplication/division parsing
        self.parse_unary()
    }

    /// Grammar: '-' unary | '!' unary | call
    fn parse_unary(&mut self) -> Result<Expr, CompilerError> {
        if let TokenType::Minus | TokenType::Not = self.peek().token_type {
            let op = match self.advance().token_type {
                TokenType::Minus => UnaryOp::Minus,
                TokenType::Not => UnaryOp::Not,
                _ => unreachable!(),
            };
            Ok(Expr::Unary {
                operator: op,
                operand: Box::new(self.parse_unary()?),
            })
        } else {
            self.parse_call()
        }
    }

    /// Grammar: primary ( '(' arguments? ')' | '[' expr ']' )*
    fn parse_call(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_primary()?;
        while let TokenType::LeftParen | TokenType::LeftBracket = self.peek().token_type {
            if self.check(&TokenType::LeftParen) {
                expr = Expr::Call {
                    callee: Box::new(expr),
                    arguments: self.parse_arguments()?,
                };
            } else {
                expr = Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(self.parse_expression()?),
                };
            }
        }
        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, CompilerError> {
        let mut arguments = Vec::new();
        while !self.check(&TokenType::RightParen) {
            arguments.push(self.parse_expression()?);
        }
        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
        Ok(arguments)
    }

    fn parse_primary(&mut self) -> Result<Expr, CompilerError> {
        match &self.peek().token_type {
            TokenType::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expr::Number(value))
            }
            TokenType::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::String(value))
            }
            TokenType::Boolean(b) => {
                let value = *b;
                self.advance();
                Ok(Expr::Boolean(value))
            }
            TokenType::Identifier(name) => {
                let value = name.clone();
                self.advance();
                Ok(Expr::Identifier(value))
            }
            TokenType::LeftParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(CompilerError::new("Expected expression".to_string())),
        }
    }

    fn parse_type(&mut self) -> Result<DataType, CompilerError> {
        let type_name = self.consume_identifier()?;
        match type_name.as_str() {
            "number" => Ok(DataType::Number),
            "str" => Ok(DataType::String),
            "bool" => Ok(DataType::Boolean),
            "array" => {
                self.consume(TokenType::LeftBracket, "Expected '[' after array type")?;
                let element_type = self.parse_type()?;
                self.consume(TokenType::RightBracket, "Expected ']' after array type")?;
                Ok(DataType::Array(Box::new(element_type)))
            }
            _ => Err(CompilerError::type_error(format!(
                "Unknown type: {}",
                type_name
            ))),
        }
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, CompilerError> {
        let mut parameters = Vec::new();

        while !self.check(&TokenType::RightParen) {
            let name = self.consume_identifier()?;
            let param_type = self
                .check(&TokenType::Colon)
                .then(|| {
                    self.advance();
                    self.parse_type()
                })
                .transpose()?;
            parameters.push(Parameter { name, param_type });
            if self.check(&TokenType::RightParen) {
                break;
            }
            self.consume(TokenType::Comma, "Expected ',' after parameter")?;
        }

        Ok(parameters)
    }

    // Utility methods
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, CompilerError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(CompilerError::new(message.to_string()))
        }
    }

    fn consume_identifier(&mut self) -> Result<String, CompilerError> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(CompilerError::new("Expected identifier".to_string()))
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), CompilerError> {
        self.consume(TokenType::Semicolon, "Expected ';' after statement")?;
        Ok(())
    }
}
