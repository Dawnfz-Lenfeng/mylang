use super::{
    expr::{BinaryOp, Expr, UnaryOp},
    stmt::Stmt,
};
use crate::{
    error::{Error, Result},
    lexer::token::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt> {
        match self.advance().token_type {
            TokenType::Let => self.var_declaration(),
            TokenType::Fn => self.function_declaration(),
            TokenType::If => self.if_statement(),
            TokenType::While => self.while_statement(),
            TokenType::For => self.for_statement(),
            TokenType::Return => self.return_statement(),
            TokenType::LeftBrace => self.block_statement(),
            TokenType::Print => self.print_statement(),
            _ => self.expression_statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier()?;
        let initializer = self
            .try_consume(TokenType::Equal)
            .then(|| self.expression())
            .transpose()?;
        self.consume_semicolon()?;
        Ok(Stmt::VarDecl { name, initializer })
    }

    fn function_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier()?;

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let parameters = self.parameters()?;
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        let body = Box::new(self.block_statement()?);

        Ok(Stmt::FuncDecl {
            name,
            parameters,
            body,
        })
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        let condition = self.expression()?;
        let then_branch = Box::new(self.block_statement()?);
        let else_branch = self
            .try_consume(TokenType::Else)
            .then(|| self.block_statement())
            .transpose()?
            .map(Box::new);
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        let condition = self.expression()?;
        let body = Box::new(self.block_statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        let initializer = match () {
            _ if self.try_consume(TokenType::Let) => Some(self.var_declaration()?),
            _ if self.try_consume(TokenType::Semicolon) => None,
            _ => Some(self.expression_statement()?),
        };

        let condition = (!self.check(TokenType::Semicolon))
            .then(|| self.expression())
            .transpose()?;
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = (!self.check(TokenType::RightParen))
            .then(|| self.expression())
            .transpose()?;

        let body = self.block_statement()?;
        let body_with_inc = match increment {
            Some(inc) => Stmt::Block(vec![body, Stmt::Expression(inc)]),
            None => body,
        };

        let while_loop = Stmt::While {
            condition: condition.unwrap_or_else(|| Expr::Boolean(true)),
            body: Box::new(body_with_inc),
        };

        Ok(match initializer {
            Some(init) => Stmt::Block(vec![init, while_loop]),
            None => while_loop,
        })
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        self.advance();
        let value = (!self.check(TokenType::Semicolon))
            .then(|| self.expression())
            .transpose()?;
        self.consume_semicolon()?;
        Ok(Stmt::Return { value })
    }

    fn block_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftBrace, "Expected '{' at start of block")?;
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) {
            statements.push(self.statement()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' at end of block")?;
        Ok(Stmt::Block(statements))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume_semicolon()?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume_semicolon()?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let mut expr = self.or()?;
        if self.try_consume(TokenType::Equal) {
            match expr {
                Expr::Variable(name) => {
                    let value = self.assignment()?;
                    expr = Expr::Assign {
                        name,
                        value: Box::new(value),
                    }
                }
                _ => {
                    return Err(Error::syntax(
                        "Invalid assignment target".to_string(),
                        self.previous().location,
                    ))
                }
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::Or], |slf| slf.and())
    }

    fn and(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::And], |slf| slf.equality())
    }

    fn equality(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::EqualEqual, TokenType::BangEqual], |slf| {
            slf.comparison()
        })
    }

    fn comparison(&mut self) -> Result<Expr> {
        self.binary(
            &[
                TokenType::LessThan,
                TokenType::LessEqual,
                TokenType::GreaterThan,
                TokenType::GreaterEqual,
            ],
            |slf| slf.term(),
        )
    }

    fn term(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::Minus, TokenType::Plus], |slf| slf.factor())
    }

    fn factor(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::Slash, TokenType::Star], |slf| slf.unary())
    }

    fn binary<F>(&mut self, ops: &[TokenType], mut next_level: F) -> Result<Expr>
    where
        F: FnMut(&mut Self) -> Result<Expr>,
    {
        let mut expr = next_level(self)?;
        while self.try_consume_any(ops) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::try_from(self.previous().token_type.clone())?,
                right: Box::new(next_level(self)?),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.try_consume_any(&[TokenType::Bang, TokenType::Minus]) {
            return Ok(Expr::Unary {
                operator: UnaryOp::try_from(self.peek().token_type.clone())?,
                operand: Box::new(self.unary()?),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;
        while self.try_consume(TokenType::LeftParen) {
            let arguments = self.arguments()?;
            expr = Expr::Call {
                callee: Box::new(expr),
                arguments,
            };
            self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.advance().token_type.clone() {
            TokenType::Number(n) => Ok(Expr::Number(n)),
            TokenType::String(s) => Ok(Expr::String(s)),
            TokenType::Boolean(b) => Ok(Expr::Boolean(b)),
            TokenType::Identifier(name) => Ok(Expr::Variable(name)),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(self.error("Expected expression".to_string())),
        }
    }

    fn parameters(&mut self) -> Result<Vec<String>> {
        let mut parameters = Vec::new();
        if self.check(TokenType::RightParen) {
            return Ok(parameters);
        }
        parameters.push(self.consume_identifier()?);
        while self.check(TokenType::Comma) {
            self.advance();
            parameters.push(self.consume_identifier()?);
        }

        Ok(parameters)
    }

    fn arguments(&mut self) -> Result<Vec<Expr>> {
        let mut arguments = Vec::new();
        if self.check(TokenType::RightParen) {
            return Ok(arguments);
        }
        arguments.push(self.expression()?);
        while self.check(TokenType::Comma) {
            self.advance();
            arguments.push(self.expression()?);
        }
        Ok(arguments)
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

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(message.to_string()))
        }
    }

    fn try_consume(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn try_consume_any(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.try_consume(token_type.clone()) {
                return true;
            }
        }
        false
    }

    fn consume_identifier(&mut self) -> Result<String> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error("Expected identifier".to_string()))
        }
    }

    fn consume_semicolon(&mut self) -> Result<()> {
        self.consume(TokenType::Semicolon, "Expected ';'")?;
        Ok(())
    }

    fn error(&self, message: String) -> Error {
        Error::syntax(message, self.peek().location)
    }
}
