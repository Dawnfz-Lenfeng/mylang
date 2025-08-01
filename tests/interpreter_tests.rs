use mylang::{
    error::Result,
    interpreter::{Interpreter, Value},
    lexer::lexer::Lexer,
    parser::{parser::Parser, Stmt},
};

#[cfg(test)]
mod interpreter_tests {
    use super::*;

    fn eval_expression(input: &str) -> Result<Value> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let mut interpreter = Interpreter::new();

        // Execute all statements except the last one
        for stmt in program.iter().take(program.len().saturating_sub(1)) {
            match stmt.accept(&mut interpreter) {
                Ok(_) => continue,
                Err(control) => return Err(control.into()),
            }
        }

        // Evaluate the last statement and return its value if it's an expression
        if let Some(last_stmt) = program.last() {
            match last_stmt {
                Stmt::Expression(expr) => expr.accept(&mut interpreter),
                _ => match last_stmt.accept(&mut interpreter) {
                    Ok(_) => Ok(Value::Nil),
                    Err(control) => Err(control.into()),
                },
            }
        } else {
            Ok(Value::Nil)
        }
    }

    fn run_program(input: &str) -> Result<()> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let mut interpreter = Interpreter::new();
        interpreter.interpret(&program)
    }

    #[test]
    fn test_number_literals() {
        let result = eval_expression("42;").unwrap();
        assert_eq!(result, Value::Number(42.0));

        let result = eval_expression("3.14;").unwrap();
        assert_eq!(result, Value::Number(3.14));
    }

    #[test]
    fn test_string_literals() {
        let result = eval_expression(r#""hello";"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_boolean_literals() {
        let result = eval_expression("true;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_expression("false;").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_arithmetic_operations() {
        let result = eval_expression("2 + 3;").unwrap();
        assert_eq!(result, Value::Number(5.0));

        let result = eval_expression("10 - 4;").unwrap();
        assert_eq!(result, Value::Number(6.0));

        let result = eval_expression("3 * 4;").unwrap();
        assert_eq!(result, Value::Number(12.0));

        let result = eval_expression("15 / 3;").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_comparison_operations() {
        let result = eval_expression("5 > 3;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_expression("2 < 8;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_expression("5 >= 5;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_expression("3 <= 2;").unwrap();
        assert_eq!(result, Value::Boolean(false));

        let result = eval_expression("42 == 42;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_expression("10 != 5;").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_logical_operations() {
        let result = eval_expression("true and false;").unwrap();
        assert_eq!(result, Value::Boolean(false));

        let result = eval_expression("true or false;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_expression("not true;").unwrap();
        assert_eq!(result, Value::Boolean(false));

        let result = eval_expression("not false;").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_unary_operations() {
        let result = eval_expression("-42;").unwrap();
        assert_eq!(result, Value::Number(-42.0));

        let result = eval_expression("-(-10);").unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_variable_declaration() {
        let program = r#"
            let x = 42;
            x;
        "#;
        let result = eval_expression(program).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_variable_assignment() {
        let program = r#"
            let x = 10;
            x = 20;
            x;
        "#;
        let result = eval_expression(program).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_variable_scoping() {
        let input = r#"
            let x = 1;
            {
                let x = 2;
                x;
            }
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_if_statement() {
        let input = r#"
            let x = 10;
            if x > 5 {
                "big";
            } else {
                "small";
            }
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::String("big".to_string()));
    }

    #[test]
    fn test_while_loop() {
        let input = r#"
            let x = 0;
            while x < 3 {
                x = x + 1;
            }
            x;
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_function_declaration_and_call() {
        let input = r#"
            fn add(a, b) {
                return a + b;
            }
            
            add(3, 4);
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_function_with_no_return() {
        let input = r#"
            fn greet(name) {
                // No explicit return
            }
            
            greet("Alice");
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_recursive_function() {
        let input = r#"
            fn factorial(n) {
                if n <= 1 {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            
            factorial(5);
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(120.0));
    }

    #[test]
    fn test_print_statement() {
        // print() should not crash and return nil
        let result = run_program("print(\"Hello, World!\");");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_undefined_variable() {
        let result = eval_expression("x + 1;");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(
                error.message.contains("Undefined variable") || error.message.contains("not found")
            );
        }
    }

    #[test]
    fn test_error_type_mismatch() {
        let result = eval_expression(r#""hello" + 42;"#);
        assert!(result.is_err());
        // The specific error depends on how type checking is implemented
    }

    #[test]
    fn test_error_function_not_found() {
        let result = eval_expression("unknown_function();");
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_expression() {
        let input = r#"
            let a = 2;
            let b = 3;
            let c = 4;
            (a + b) * c - a;
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(18.0)); // (2 + 3) * 4 - 2 = 20 - 2 = 18
    }

    #[test]
    fn test_nested_function_calls() {
        let input = r#"
            fn double(x) {
                return x * 2;
            }
            
            fn add_one(x) {
                return x + 1;
            }
            
            double(add_one(5));
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(12.0)); // double(add_one(5)) = double(6) = 12
    }

    #[test]
    fn test_truthiness() {
        // Test how different values are treated in boolean contexts
        let result = eval_expression("if 0 { \"truthy\"; } else { \"falsy\"; }").unwrap();
        assert_eq!(result, Value::String("falsy".to_string()));

        let result = eval_expression("if 1 { \"truthy\"; } else { \"falsy\"; }").unwrap();
        assert_eq!(result, Value::String("truthy".to_string()));

        let result = eval_expression(r#"if "" { "truthy"; } else { "falsy"; }"#).unwrap();
        assert_eq!(result, Value::String("falsy".to_string()));

        let result = eval_expression(r#"if "hello" { "truthy"; } else { "falsy"; }"#).unwrap();
        assert_eq!(result, Value::String("truthy".to_string()));
    }

    #[test]
    fn test_string_concatenation() {
        let result = eval_expression(r#""hello" + " " + "world";"#).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_parenthesized_expressions() {
        let result = eval_expression("(1 + 2) * 3;").unwrap();
        assert_eq!(result, Value::Number(9.0));
    }

    #[test]
    fn test_operator_precedence() {
        let result = eval_expression("2 + 3 * 4;").unwrap();
        assert_eq!(result, Value::Number(14.0)); // Should be 2 + (3 * 4) = 14

        let result = eval_expression("(2 + 3) * 4;").unwrap();
        assert_eq!(result, Value::Number(20.0)); // Should be (2 + 3) * 4 = 20
    }

    #[test]
    fn test_assignment_in_expression() {
        let input = r#"
            let x = 5;
            let y = x = 10;
            y;
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_block_scoping() {
        let input = r#"
            let outer = "outer";
            {
                let inner = "inner";
                inner;
            }
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::String("inner".to_string()));
    }

    #[test]
    fn test_nested_if_statements() {
        let input = r#"
            let x = 5;
            let y = 3;
            if x > 0 {
                if y > 0 {
                    "both positive";
                } else {
                    "x positive, y not positive";
                }
            } else {
                "x not positive";
            }
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::String("both positive".to_string()));
    }

    #[test]
    fn test_function_closure() {
        let input = r#"
            let outer_var = 42;
            fn inner_func() {
                return outer_var;
            }
            inner_func();
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_return_statement() {
        let input = r#"
            fn early_return(x) {
                if x > 0 {
                    return "positive";
                }
                return "not positive";
            }
            early_return(5);
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::String("positive".to_string()));
    }

    #[test]
    fn test_empty_function() {
        let input = r#"
            fn empty() {
            }
            empty();
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_function_parameters() {
        let input = r#"
            fn test_params(a, b, c) {
                return a + b + c;
            }
            test_params(1, 2, 3);
        "#;
        let result = eval_expression(input).unwrap();
        assert_eq!(result, Value::Number(6.0));
    }
}
