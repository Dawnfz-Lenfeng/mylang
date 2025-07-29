use crate::ast::{BinaryOp, DataType, Expr, Parameter, Program, Stmt, UnaryOp};
use crate::error::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: DataType,
    pub is_mutable: bool,
    pub is_initialized: bool,
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, symbol: Symbol) -> Result<(), CompilerError> {
        let current_scope = self.scopes.last_mut().unwrap();

        if current_scope.contains_key(&name) {
            return Err(CompilerError::new(format!(
                "Variable '{}' already defined in current scope",
                name
            )));
        }

        current_scope.insert(name, symbol);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn assign(&mut self, name: &str, value_type: DataType) -> Result<(), CompilerError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                if !symbol.is_mutable {
                    return Err(CompilerError::semantic_error(format!(
                        "Cannot assign to immutable variable '{}'",
                        name
                    )));
                }

                if symbol.symbol_type != value_type {
                    return Err(CompilerError::semantic_error(format!(
                        "Type mismatch: expected {:?}, found {:?}",
                        symbol.symbol_type, value_type
                    )));
                }

                symbol.is_initialized = true;
                return Ok(());
            }
        }

        Err(CompilerError::semantic_error(format!(
            "Undefined variable '{}'",
            name
        )))
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    current_function_return_type: Option<DataType>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            current_function_return_type: None,
        }
    }

    /// Convert type expression to DataType
    fn expr_to_type(&self, expr: &Expr) -> Result<DataType, CompilerError> {
        match expr {
            Expr::Identifier(name) => match name.as_str() {
                "number" => Ok(DataType::Number),
                "str" => Ok(DataType::String),
                "bool" => Ok(DataType::Boolean),
                "void" => Ok(DataType::Void),
                _ => Err(CompilerError::type_error(format!("Unknown type: {}", name))),
            },
            Expr::Index { array, index } => {
                if let Expr::Identifier(array_name) = array.as_ref() {
                    if array_name == "array" {
                        let element_type = self.expr_to_type(index)?;
                        return Ok(DataType::Array(Box::new(element_type)));
                    }
                }
                Err(CompilerError::type_error(
                    "Invalid type expression".to_string(),
                ))
            }
            _ => Err(CompilerError::type_error(
                "Invalid type expression".to_string(),
            )),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), CompilerError> {
        for statement in &program.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), CompilerError> {
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                initializer,
                is_mutable,
            } => self.analyze_var_declaration(name, type_annotation, initializer, *is_mutable),
            Stmt::FuncDecl {
                name,
                parameters,
                return_type,
                body,
            } => self.analyze_function_declaration(name, parameters, return_type, body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.analyze_if_statement(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.analyze_while_statement(condition, body),
            Stmt::For {
                name,
                collection,
                body,
            } => self.analyze_for_statement(name, collection, body),
            Stmt::Return { value } => self.analyze_return_statement(value),
            Stmt::Expression(expr) => {
                self.analyze_expression(expr)?;
                Ok(())
            }
            Stmt::Block(statements) => self.analyze_block(statements),
        }
    }

    fn analyze_var_declaration(
        &mut self,
        name: &str,
        type_annotation: &Option<Expr>,
        initializer: &Option<Expr>,
        is_mutable: bool,
    ) -> Result<(), CompilerError> {
        let var_type = if let Some(init_expr) = initializer {
            let expr_type = self.analyze_expression(init_expr)?;

            if let Some(annotation_expr) = type_annotation {
                let annotation = self.expr_to_type(annotation_expr)?;
                if annotation != expr_type {
                    return Err(CompilerError::semantic_error(format!(
                        "Type mismatch in variable declaration: expected {:?}, found {:?}",
                        annotation, expr_type
                    )));
                }
                annotation
            } else {
                expr_type
            }
        } else if let Some(annotation_expr) = type_annotation {
            self.expr_to_type(annotation_expr)?
        } else {
            return Err(CompilerError::semantic_error(
                "Variable declaration must have either type annotation or initializer".to_string(),
            ));
        };

        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: var_type,
            is_mutable,
            is_initialized: initializer.is_some(),
        };

        self.symbol_table.define(name.to_string(), symbol)
    }

    fn analyze_function_declaration(
        &mut self,
        name: &str,
        parameters: &[Parameter],
        return_type: &Option<Expr>,
        body: &[Stmt],
    ) -> Result<(), CompilerError> {
        // Define function in symbol table
        let param_types: Vec<DataType> = parameters
            .iter()
            .map(|param| {
                param
                    .param_type
                    .as_ref()
                    .map(|type_expr| self.expr_to_type(type_expr))
                    .transpose()
                    .map(|result| result.unwrap_or(DataType::Any))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let ret_type = if let Some(return_type_expr) = return_type {
            self.expr_to_type(return_type_expr)?
        } else {
            DataType::Void
        };

        let function_type = DataType::Function(param_types.clone(), Box::new(ret_type.clone()));

        let function_symbol = Symbol {
            name: name.to_string(),
            symbol_type: function_type,
            is_mutable: false,
            is_initialized: true,
        };

        self.symbol_table
            .define(name.to_string(), function_symbol)?;

        // Analyze function body in new scope
        self.symbol_table.enter_scope();
        self.current_function_return_type = Some(ret_type);

        // Add parameters to scope
        for (param, param_type) in parameters.iter().zip(param_types.iter()) {
            let param_symbol = Symbol {
                name: param.name.clone(),
                symbol_type: param_type.clone(),
                is_mutable: true,
                is_initialized: true,
            };
            self.symbol_table.define(param.name.clone(), param_symbol)?;
        }

        // Analyze body
        for stmt in body {
            self.analyze_statement(stmt)?;
        }

        self.current_function_return_type = None;
        self.symbol_table.exit_scope();
        Ok(())
    }

    fn analyze_if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &Option<Vec<Stmt>>,
    ) -> Result<(), CompilerError> {
        let condition_type = self.analyze_expression(condition)?;
        if condition_type != DataType::Boolean {
            return Err(CompilerError::semantic_error(format!(
                "If condition must be boolean, found {:?}",
                condition_type
            )));
        }

        self.symbol_table.enter_scope();
        for stmt in then_branch {
            self.analyze_statement(stmt)?;
        }
        self.symbol_table.exit_scope();

        if let Some(else_stmts) = else_branch {
            self.analyze_block(else_stmts)?;
        }

        Ok(())
    }

    fn analyze_while_statement(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
    ) -> Result<(), CompilerError> {
        let condition_type = self.analyze_expression(condition)?;
        if condition_type != DataType::Boolean {
            return Err(CompilerError::semantic_error(format!(
                "While condition must be boolean, found {:?}",
                condition_type
            )));
        }

        self.analyze_block(body)
    }

    fn analyze_for_statement(
        &mut self,
        name: &str,
        collection: &Expr,
        body: &[Stmt],
    ) -> Result<(), CompilerError> {
        self.symbol_table.enter_scope();

        let collection_type = self.analyze_expression(collection)?;
        if collection_type != DataType::Array(Box::new(DataType::Any)) {
            return Err(CompilerError::semantic_error(format!(
                "For loop collection must be an array, found {:?}",
                collection_type
            )));
        }

        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: collection_type,
            is_mutable: false,
            is_initialized: true,
        };

        self.symbol_table.define(name.to_string(), symbol)?;
        for stmt in body {
            self.analyze_statement(stmt)?;
        }

        self.symbol_table.exit_scope();
        Ok(())
    }

    fn analyze_return_statement(&mut self, value: &Option<Expr>) -> Result<(), CompilerError> {
        let return_type = if let Some(expr) = value {
            self.analyze_expression(expr)?
        } else {
            DataType::Void
        };

        let expected_type = self
            .current_function_return_type
            .clone()
            .unwrap_or(DataType::Void);

        if return_type != expected_type {
            return Err(CompilerError::semantic_error(format!(
                "Return type mismatch: expected {:?}, found {:?}",
                expected_type, return_type
            )));
        }

        Ok(())
    }

    fn analyze_block(&mut self, statements: &[Stmt]) -> Result<(), CompilerError> {
        self.symbol_table.enter_scope();
        for stmt in statements {
            self.analyze_statement(stmt)?;
        }
        self.symbol_table.exit_scope();
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Result<DataType, CompilerError> {
        match expr {
            Expr::Number(_) => Ok(DataType::Number),
            Expr::String(_) => Ok(DataType::String),
            Expr::Boolean(_) => Ok(DataType::Boolean),
            Expr::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.get(name) {
                    if !symbol.is_initialized {
                        return Err(CompilerError::semantic_error(format!(
                            "Use of uninitialized variable '{}'",
                            name
                        )));
                    }
                    Ok(symbol.symbol_type.clone())
                } else {
                    Err(CompilerError::semantic_error(format!(
                        "Undefined variable '{}'",
                        name
                    )))
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => self.analyze_binary_expression(left, operator, right),
            Expr::Unary { operator, operand } => self.analyze_unary_expression(operator, operand),
            Expr::Call { callee, arguments } => self.analyze_call_expression(callee, arguments),
            Expr::Index { array, index } => self.analyze_index_expression(array, index),
            Expr::Assign { name, value } => {
                let value_type = self.analyze_expression(value)?;
                self.symbol_table.assign(name, value_type.clone())?;
                Ok(value_type)
            }
            Expr::Array { elements } => {
                let element_type = self.analyze_expression(elements.first().unwrap())?;
                for element in elements {
                    let element_type = self.analyze_expression(element)?;
                    if element_type != element_type {
                        return Err(CompilerError::semantic_error(format!(
                            "All elements of array must be of the same type, found {:?}",
                            element_type
                        )));
                    }
                }
                Ok(DataType::Array(Box::new(element_type)))
            }
        }
    }

    fn analyze_binary_expression(
        &mut self,
        left: &Expr,
        operator: &BinaryOp,
        right: &Expr,
    ) -> Result<DataType, CompilerError> {
        let left_type = self.analyze_expression(left)?;
        let right_type = self.analyze_expression(right)?;

        match operator {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::BitAnd
            | BinaryOp::BitOr => {
                if left_type == DataType::Number && right_type == DataType::Number {
                    Ok(DataType::Number)
                } else {
                    Err(CompilerError::semantic_error(
                        "Arithmetic operations require numeric operands".to_string(),
                    ))
                }
            }
            BinaryOp::Equal | BinaryOp::NotEqual => {
                if left_type == right_type {
                    Ok(DataType::Boolean)
                } else {
                    Err(CompilerError::semantic_error(
                        "Comparison requires operands of same type".to_string(),
                    ))
                }
            }
            BinaryOp::LessThan
            | BinaryOp::LessEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterEqual => {
                if left_type == DataType::Number && right_type == DataType::Number {
                    Ok(DataType::Boolean)
                } else {
                    Err(CompilerError::semantic_error(
                        "Comparison operations require numeric operands".to_string(),
                    ))
                }
            }
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                if left_type == DataType::Boolean && right_type == DataType::Boolean {
                    Ok(DataType::Boolean)
                } else {
                    Err(CompilerError::semantic_error(
                        "Logical operations require boolean operands".to_string(),
                    ))
                }
            }
            BinaryOp::Assign => {
                if left_type == right_type {
                    Ok(left_type)
                } else {
                    Err(CompilerError::semantic_error(
                        "Assignment requires operands of same type".to_string(),
                    ))
                }
            }
        }
    }

    fn analyze_unary_expression(
        &mut self,
        operator: &UnaryOp,
        operand: &Expr,
    ) -> Result<DataType, CompilerError> {
        let operand_type = self.analyze_expression(operand)?;

        match operator {
            UnaryOp::Minus => {
                if operand_type == DataType::Number {
                    Ok(DataType::Number)
                } else {
                    Err(CompilerError::semantic_error(
                        "Unary minus requires numeric operand".to_string(),
                    ))
                }
            }
            UnaryOp::Not => {
                if operand_type == DataType::Boolean {
                    Ok(DataType::Boolean)
                } else {
                    Err(CompilerError::semantic_error(
                        "Logical not requires boolean operand".to_string(),
                    ))
                }
            }
        }
    }

    fn analyze_call_expression(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
    ) -> Result<DataType, CompilerError> {
        let callee_type = self.analyze_expression(callee)?;

        if let DataType::Function(param_types, return_type) = callee_type {
            if arguments.len() != param_types.len() {
                return Err(CompilerError::semantic_error(format!(
                    "Function expects {} arguments, got {}",
                    param_types.len(),
                    arguments.len()
                )));
            }

            for (i, arg) in arguments.iter().enumerate() {
                let arg_type = self.analyze_expression(arg)?;
                if arg_type != param_types[i] {
                    return Err(CompilerError::semantic_error(format!(
                        "Argument {} type mismatch: expected {:?}, found {:?}",
                        i, param_types[i], arg_type
                    )));
                }
            }

            Ok(*return_type)
        } else {
            Err(CompilerError::semantic_error(
                "Cannot call non-function value".to_string(),
            ))
        }
    }

    fn analyze_index_expression(
        &mut self,
        array: &Expr,
        index: &Expr,
    ) -> Result<DataType, CompilerError> {
        let array_type = self.analyze_expression(array)?;
        let index_type = self.analyze_expression(index)?;

        if index_type != DataType::Number {
            return Err(CompilerError::semantic_error(
                "Array index must be numeric".to_string(),
            ));
        }

        if let DataType::Array(element_type) = array_type {
            Ok(*element_type)
        } else {
            Err(CompilerError::semantic_error(
                "Cannot index non-array value".to_string(),
            ))
        }
    }
}
