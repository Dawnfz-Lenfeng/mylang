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
            statements.push(self.stmt()?);
        }

        Ok(statements)
    }

    fn stmt(&mut self) -> Result<Stmt> {
        match self.peek().token_type {
            TokenType::Let => self.var_decl(),
            TokenType::Fn => self.func_decl(),
            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::For => self.for_stmt(),
            TokenType::Return => self.return_stmt(),
            TokenType::LeftBrace => self.block_stmt(),
            TokenType::Print => self.print_stmt(),
            TokenType::Break => self.break_stmt(),
            TokenType::Continue => self.continue_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn var_decl(&mut self) -> Result<Stmt> {
        self.advance();
        let name = self.consume_identifier()?;
        let initializer = self
            .try_consume(TokenType::Equal)
            .is_some()
            .then(|| self.expr())
            .transpose()?;
        self.consume_semicolon()?;
        Ok(Stmt::VarDecl { name, initializer })
    }

    fn func_decl(&mut self) -> Result<Stmt> {
        self.advance();
        let name = self.consume_identifier()?;

        self.consume(TokenType::LeftParen, "expected '(' after function name")?;
        let params = self.parameters()?;
        self.consume(TokenType::RightParen, "expected ')' after parameters")?;

        let body = Box::new(self.block_stmt()?);

        Ok(Stmt::FuncDecl { name, params, body })
    }

    fn if_stmt(&mut self) -> Result<Stmt> {
        self.advance();
        let condition = self.expr()?;
        let then_branch = Box::new(self.block_stmt()?);
        let else_branch = self
            .try_consume(TokenType::Else)
            .is_some()
            .then(|| {
                if self.check(&TokenType::If) {
                    self.if_stmt()
                } else {
                    self.block_stmt()
                }
            })
            .transpose()?
            .map(Box::new);

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        self.advance();
        let condition = self.expr()?;
        let body = Box::new(self.block_stmt()?);
        Ok(Stmt::While { condition, body })
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        self.advance();
        let initializer = match self.peek().token_type {
            TokenType::Let => Some(self.var_decl()?),
            TokenType::Semicolon => {
                self.advance();
                None
            }
            _ => Some(self.expr_stmt()?),
        };

        let condition = (!self.check(&TokenType::Semicolon))
            .then(|| self.expr())
            .transpose()?
            .unwrap_or_else(|| Expr::Boolean(true));
        self.consume(TokenType::Semicolon, "expect ';' after loop condition")?;

        let increment = (!self.check(&TokenType::RightParen))
            .then(|| self.expr())
            .transpose()?;

        let body = self.block_stmt()?;
        let body_with_inc = match increment {
            Some(inc) => Stmt::Block(vec![body, Stmt::Expression(inc)]),
            None => body,
        };

        let while_loop = Stmt::While {
            condition,
            body: Box::new(body_with_inc),
        };

        Ok(match initializer {
            Some(init) => Stmt::Block(vec![init, while_loop]),
            None => while_loop,
        })
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        self.advance();
        let value = (!self.check(&TokenType::Semicolon))
            .then(|| self.expr())
            .transpose()?;
        self.consume_semicolon()?;
        Ok(Stmt::Return { value })
    }

    fn break_stmt(&mut self) -> Result<Stmt> {
        self.advance();
        self.consume_semicolon()?;
        Ok(Stmt::Break)
    }

    fn continue_stmt(&mut self) -> Result<Stmt> {
        self.advance();
        self.consume_semicolon()?;
        Ok(Stmt::Continue)
    }

    fn block_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftBrace, "expected '{' at start of block")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            statements.push(self.stmt()?);
        }
        self.consume(TokenType::RightBrace, "expected '}' at end of block")?;
        Ok(Stmt::Block(statements))
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        self.advance();
        let exprs = self.arguments()?;
        self.consume_semicolon()?;
        Ok(Stmt::Print(exprs))
    }

    fn expr_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        self.consume_semicolon()?;
        Ok(Stmt::Expression(expr))
    }

    fn expr(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let mut expr = self.or()?;
        if let Some(token) = self.try_consume_any(&[
            TokenType::Equal,
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::StarEqual,
            TokenType::SlashEqual,
        ]) {
            match expr {
                Expr::Variable(name) => {
                    let op_type = token.token_type.clone();
                    let value = self.assignment()?;
                    expr = match op_type {
                        TokenType::Equal => Expr::Assign {
                            name,
                            value: Box::new(value),
                        },
                        TokenType::PlusEqual => Expr::Assign {
                            name: name.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable(name)),
                                operator: BinaryOp::Add,
                                right: Box::new(value),
                            }),
                        },
                        TokenType::MinusEqual => Expr::Assign {
                            name: name.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable(name)),
                                operator: BinaryOp::Subtract,
                                right: Box::new(value),
                            }),
                        },
                        TokenType::StarEqual => Expr::Assign {
                            name: name.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable(name)),
                                operator: BinaryOp::Multiply,
                                right: Box::new(value),
                            }),
                        },
                        TokenType::SlashEqual => Expr::Assign {
                            name: name.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable(name)),
                                operator: BinaryOp::Divide,
                                right: Box::new(value),
                            }),
                        },
                        _ => unreachable!(),
                    }
                }
                Expr::Index { array, index } => {
                    let op_type = token.token_type.clone();
                    let value = self.assignment()?;
                    expr = match op_type {
                        TokenType::Equal => Expr::IndexAssign {
                            array,
                            index,
                            value: Box::new(value),
                        },
                        TokenType::PlusEqual => Expr::IndexAssign {
                            array: array.clone(),
                            index: index.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Index { array, index }),
                                operator: BinaryOp::Add,
                                right: Box::new(value),
                            }),
                        },
                        TokenType::MinusEqual => Expr::IndexAssign {
                            array: array.clone(),
                            index: index.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Index { array, index }),
                                operator: BinaryOp::Subtract,
                                right: Box::new(value),
                            }),
                        },
                        TokenType::StarEqual => Expr::IndexAssign {
                            array: array.clone(),
                            index: index.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Index { array, index }),
                                operator: BinaryOp::Multiply,
                                right: Box::new(value),
                            }),
                        },
                        TokenType::SlashEqual => Expr::IndexAssign {
                            array: array.clone(),
                            index: index.clone(),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Index { array, index }),
                                operator: BinaryOp::Divide,
                                right: Box::new(value),
                            }),
                        },
                        _ => unreachable!(),
                    }
                }
                _ => {
                    return Err(Error::syntax(
                        "invalid assignment target".to_string(),
                        token.location,
                    ))
                }
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::Or], Self::and)
    }

    fn and(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::And], Self::equality)
    }

    fn equality(&mut self) -> Result<Expr> {
        self.binary(
            &[TokenType::EqualEqual, TokenType::BangEqual],
            Self::comparison,
        )
    }

    fn comparison(&mut self) -> Result<Expr> {
        self.binary(
            &[
                TokenType::LessThan,
                TokenType::LessEqual,
                TokenType::GreaterThan,
                TokenType::GreaterEqual,
            ],
            Self::term,
        )
    }

    fn term(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::Minus, TokenType::Plus], Self::factor)
    }

    fn factor(&mut self) -> Result<Expr> {
        self.binary(&[TokenType::Slash, TokenType::Star], Self::unary)
    }

    fn binary<F>(&mut self, ops: &[TokenType], mut next_level: F) -> Result<Expr>
    where
        F: FnMut(&mut Self) -> Result<Expr>,
    {
        let mut expr = next_level(self)?;
        while let Some(token) = self.try_consume_any(ops) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::try_from(token.token_type.clone())?,
                right: Box::new(next_level(self)?),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Some(token) = self.try_consume_any(&[TokenType::Bang, TokenType::Minus]) {
            return Ok(Expr::Unary {
                operator: UnaryOp::try_from(token.token_type.clone())?,
                operand: Box::new(self.unary()?),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.try_consume(TokenType::LeftParen).is_some() {
                let arguments = self.arguments()?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    arguments,
                };
                self.consume(TokenType::RightParen, "expected ')' after arguments")?;
            } else if self.try_consume(TokenType::LeftBracket).is_some() {
                let index = self.expr()?;
                expr = Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
                self.consume(TokenType::RightBracket, "expected ']' after array index")?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.advance();

        match &token.token_type {
            TokenType::Number(n) => Ok(Expr::Number(*n)),
            TokenType::String(s) => Ok(Expr::String(s.clone())),
            TokenType::Boolean(b) => Ok(Expr::Boolean(*b)),
            TokenType::Identifier(name) => Ok(Expr::Variable(name.clone())),
            TokenType::Nil => Ok(Expr::Nil),
            TokenType::LeftParen => {
                let expr = self.expr()?;
                self.consume(TokenType::RightParen, "expected ')' after expression")?;
                Ok(expr)
            }
            TokenType::LeftBracket => {
                let elements = self.arguments()?;
                self.consume(TokenType::RightBracket, "expected ']' after array elements")?;
                Ok(Expr::Array(elements))
            }
            _ => {
                let expected = "number, string, boolean, identifier, '(' or '['";
                Err(Error::syntax(
                    format!("expected {}, found {:?}", expected, token.token_type),
                    token.location,
                ))
            }
        }
    }

    fn parameters(&mut self) -> Result<Vec<String>> {
        let mut parameters = Vec::new();
        if self.check(&TokenType::RightParen) {
            return Ok(parameters);
        }
        parameters.push(self.consume_identifier()?);
        while self.check(&TokenType::Comma) {
            self.advance();
            parameters.push(self.consume_identifier()?);
        }

        Ok(parameters)
    }

    fn arguments(&mut self) -> Result<Vec<Expr>> {
        let mut arguments = Vec::new();
        if self.check(&TokenType::RightParen) {
            return Ok(arguments);
        }
        arguments.push(self.expr()?);
        while self.check(&TokenType::Comma) {
            self.advance();
            arguments.push(self.expr()?);
        }
        Ok(arguments)
    }

    // Utility methods
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.current];
        if !self.is_at_end() {
            self.current += 1;
        }
        token
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == *token_type
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(message.to_string()))
        }
    }

    fn try_consume(&mut self, token_type: TokenType) -> Option<&Token> {
        if self.check(&token_type) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn try_consume_any(&mut self, token_types: &[TokenType]) -> Option<&Token> {
        for token_type in token_types {
            if self.check(token_type) {
                return Some(self.advance());
            }
        }
        None
    }

    fn consume_identifier(&mut self) -> Result<String> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error("expected identifier".to_string()))
        }
    }

    fn consume_semicolon(&mut self) -> Result<()> {
        self.consume(TokenType::Semicolon, "expected ';'")?;
        Ok(())
    }

    fn error(&self, message: String) -> Error {
        Error::syntax(message, self.peek().location)
    }
}
