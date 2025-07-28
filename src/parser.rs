use crate::ast::{BinaryOp, DataType, Expr, Parameter, Program, Stmt, UnaryOp};
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
            program.add_statement(self.parse_statement()?);
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
        let parameters = self.parse_parameters()?;
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
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume_semicolon()?;
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
        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if self.check(&TokenType::RightParen) {
            self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
            return Ok(parameters);
        }
        parameters.push(self.parse_parameter()?);

        while self.check(&TokenType::Comma) {
            self.advance();
            parameters.push(self.parse_parameter()?);
        }
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        Ok(parameters)
    }

    fn parse_parameter(&mut self) -> Result<Parameter, CompilerError> {
        let name = self.consume_identifier()?;
        let param_type = self
            .check(&TokenType::Colon)
            .then(|| {
                self.advance();
                self.parse_type()
            })
            .transpose()?;
        Ok(Parameter { name, param_type })
    }

    /// Grammar: assignment
    fn parse_expression(&mut self) -> Result<Expr, CompilerError> {
        self.parse_assignment()
    }

    /// Grammar: or ('=' or)*
    fn parse_assignment(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_or()?;
        if self.check(&TokenType::Assign) {
            let op = match self.advance().token_type {
                TokenType::Assign => BinaryOp::Assign,
                _ => unreachable!(),
            };
            let right = self.parse_assignment()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Grammar: and ('||' and)*
    fn parse_or(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_and()?;
        while matches!(self.peek().token_type, TokenType::Or) {
            let op = match self.advance().token_type {
                TokenType::Or => BinaryOp::Or,
                _ => unreachable!(),
            };
            let right = self.parse_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Grammar: equality ('&&' equality)*
    fn parse_and(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_equality()?;
        while matches!(self.peek().token_type, TokenType::And) {
            let op = match self.advance().token_type {
                TokenType::And => BinaryOp::And,
                _ => unreachable!(),
            };
            let right = self.parse_equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Grammar: comparison ('==' comparison | '!=' comparison)*
    fn parse_equality(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_comparison()?;
        while matches!(
            self.peek().token_type,
            TokenType::Equal | TokenType::NotEqual
        ) {
            let op = match self.advance().token_type {
                TokenType::Equal => BinaryOp::Equal,
                TokenType::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Grammar: term ('<' term | '>' term | '<=' term | '>=' term)*
    fn parse_comparison(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_term()?;
        while matches!(
            self.peek().token_type,
            TokenType::LessThan
                | TokenType::GreaterThan
                | TokenType::LessEqual
                | TokenType::GreaterEqual
        ) {
            let op = match self.advance().token_type {
                TokenType::LessThan => BinaryOp::LessThan,
                TokenType::GreaterThan => BinaryOp::GreaterThan,
                TokenType::LessEqual => BinaryOp::LessEqual,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Grammar: factor '+' term | '-' term
    fn parse_term(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_factor()?;
        while matches!(self.peek().token_type, TokenType::Plus | TokenType::Minus) {
            let op = match self.advance().token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Grammar: unary '*' factor | '/' factor | '%' factor
    fn parse_factor(&mut self) -> Result<Expr, CompilerError> {
        let mut expr = self.parse_unary()?;
        while matches!(
            self.peek().token_type,
            TokenType::Asterisk | TokenType::Slash | TokenType::Percent
        ) {
            let op = match self.advance().token_type {
                TokenType::Asterisk => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Grammar: '-' unary | '!' unary | call
    fn parse_unary(&mut self) -> Result<Expr, CompilerError> {
        if matches!(self.peek().token_type, TokenType::Minus | TokenType::Not) {
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
        while matches!(
            self.peek().token_type,
            TokenType::LeftParen | TokenType::LeftBracket
        ) {
            expr = if self.check(&TokenType::LeftParen) {
                Expr::Call {
                    callee: Box::new(expr),
                    arguments: self.parse_arguments()?,
                }
            } else {
                self.advance();
                let index = self.parse_expression()?;
                self.consume(TokenType::RightBracket, "Expected ']' after array index")?;
                Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                }
            };
        }
        Ok(expr)
    }

    /// Grammar: expression ( ',' expression )*
    fn parse_arguments(&mut self) -> Result<Vec<Expr>, CompilerError> {
        self.advance(); // consume '('
        let mut arguments = Vec::new();
        if self.check(&TokenType::RightParen) {
            self.advance(); // consume ')'
            return Ok(arguments);
        }
        arguments.push(self.parse_expression()?);
        while self.check(&TokenType::Comma) {
            self.advance();
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
            _ => Err(CompilerError::syntax_error(
                "Expected expression".to_string(),
                self.peek().line,
                self.peek().column,
            )),
        }
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
            self.peek().token_type == *token_type
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, CompilerError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(CompilerError::syntax_error(
                message.to_string(),
                self.peek().line,
                self.peek().column,
            ))
        }
    }

    fn consume_identifier(&mut self) -> Result<String, CompilerError> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(CompilerError::syntax_error(
                "Expected identifier".to_string(),
                self.peek().line,
                self.peek().column,
            ))
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), CompilerError> {
        self.consume(TokenType::Semicolon, "Expected ';' after statement")?;
        Ok(())
    }
}
