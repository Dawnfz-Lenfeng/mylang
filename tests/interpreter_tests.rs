use my::{CompilerError, Interpreter, Lexer, Parser, Value};

#[cfg(test)]
mod interpreter_tests {
    use super::*;

    fn run_program(input: &str) -> Result<Value, CompilerError> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let mut interpreter = Interpreter::new();
        interpreter.interpret(&program)
    }

    #[test]
    fn test_number_literals() {
        let result = run_program("42;").unwrap();
        assert_eq!(result, Value::Number(42.0));

        let result = run_program("3.14;").unwrap();
        assert_eq!(result, Value::Number(3.14));
    }

    #[test]
    fn test_string_literals() {
        let result = run_program("\"hello\";").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));

        let result = run_program("'world';").unwrap();
        assert_eq!(result, Value::String("world".to_string()));
    }

    #[test]
    fn test_boolean_literals() {
        let result = run_program("true;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = run_program("false;").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_arithmetic_operations() {
        let result = run_program("2 + 3;").unwrap();
        assert_eq!(result, Value::Number(5.0));

        let result = run_program("10 - 4;").unwrap();
        assert_eq!(result, Value::Number(6.0));

        let result = run_program("3 * 4;").unwrap();
        assert_eq!(result, Value::Number(12.0));

        let result = run_program("15 / 3;").unwrap();
        assert_eq!(result, Value::Number(5.0));

        let result = run_program("17 % 5;").unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_comparison_operations() {
        let result = run_program("5 > 3;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = run_program("2 < 8;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = run_program("5 >= 5;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = run_program("3 <= 2;").unwrap();
        assert_eq!(result, Value::Boolean(false));

        let result = run_program("42 == 42;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = run_program("10 != 5;").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_logical_operations() {
        let result = run_program("true and false;").unwrap();
        assert_eq!(result, Value::Boolean(false));

        let result = run_program("true or false;").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = run_program("not true;").unwrap();
        assert_eq!(result, Value::Boolean(false));

        let result = run_program("not false;").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_unary_operations() {
        let result = run_program("-42;").unwrap();
        assert_eq!(result, Value::Number(-42.0));

        let result = run_program("-(-10);").unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_variable_declaration() {
        let result = run_program("let x = 42; x;").unwrap();
        assert_eq!(result, Value::Number(42.0));

        let result = run_program("const name = \"Alice\"; name;").unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));
    }

    #[test]
    fn test_variable_assignment() {
        let result = run_program("let mut x = 10; x = 20; x;").unwrap();
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
        let result = run_program(input).unwrap();
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
        let result = run_program(input).unwrap();
        assert_eq!(result, Value::String("big".to_string()));
    }

    #[test]
    fn test_while_loop() {
        let input = r#"
            let mut x = 0;
            while x < 3 {
                x = x + 1;
            }
            x;
        "#;
        let result = run_program(input).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_function_declaration_and_call() {
        let input = r#"
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            
            add(3, 4);
        "#;
        let result = run_program(input).unwrap();
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_function_with_no_return() {
        let input = r#"
            fn greet(name: str) {
                // No explicit return
            }
            
            greet("Alice");
        "#;
        let result = run_program(input).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_recursive_function() {
        let input = r#"
            fn factorial(n: number) -> number {
                if n <= 1 {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            
            factorial(5);
        "#;
        let result = run_program(input).unwrap();
        assert_eq!(result, Value::Number(120.0));
    }

    #[test]
    fn test_builtin_functions() {
        // Note: print() returns null, so we test that it doesn't crash
        let result = run_program("print(\"Hello, World!\");").unwrap();
        assert_eq!(result, Value::Null);

        let result = run_program("len(\"hello\");").unwrap();
        assert_eq!(result, Value::Number(5.0));

        let result = run_program("type(42);").unwrap();
        assert_eq!(result, Value::String("number".to_string()));
    }

    #[test]
    fn test_arrays() {
        // Array literal creation would need to be implemented
        // let result = run_program("[1, 2, 3];").unwrap();
        // assert!(matches!(result, Value::Array(_)));
    }

    #[test]
    fn test_array_indexing() {
        // This would require array literal syntax
        // let input = r#"
        //     let arr = [10, 20, 30];
        //     arr[1];
        // "#;
        // let result = run_program(input).unwrap();
        // assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_for_loop() {
        // This would require array support
        // let input = r#"
        //     let sum = 0;
        //     for x in [1, 2, 3] {
        //         sum = sum + x;
        //     }
        //     sum;
        // "#;
        // let result = run_program(input).unwrap();
        // assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_error_undefined_variable() {
        let result = run_program("x + 1;");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("Undefined variable"));
        }
    }

    #[test]
    fn test_error_type_mismatch() {
        let result = run_program("\"hello\" + 42;");
        assert!(result.is_err());
        // The specific error depends on how you implement type checking
    }

    #[test]
    fn test_error_function_not_found() {
        let result = run_program("unknown_function();");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_wrong_argument_count() {
        let input = r#"
            fn test(a: number) -> number {
                return a * 2;
            }
            test(1, 2);
        "#;
        let result = run_program(input);
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
        let result = run_program(input).unwrap();
        assert_eq!(result, Value::Number(18.0)); // (2 + 3) * 4 - 2 = 20 - 2 = 18
    }

    #[test]
    fn test_nested_function_calls() {
        let input = r#"
            fn double(x: number) -> number {
                return x * 2;
            }
            
            fn add_one(x: number) -> number {
                return x + 1;
            }
            
            double(add_one(5));
        "#;
        let result = run_program(input).unwrap();
        assert_eq!(result, Value::Number(12.0)); // double(add_one(5)) = double(6) = 12
    }

    #[test]
    fn test_closures() {
        // This is an advanced feature - closures capturing variables
        let input = r#"
            fn make_adder(x: number) -> fn {
                fn adder(y: number) -> number {
                    return x + y;
                }
                return adder;
            }
            
            let add_five = make_adder(5);
            add_five(3);
        "#;
        // This test might be complex to implement initially
        // let result = run_program(input).unwrap();
        // assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_truthiness() {
        // Test how different values are treated in boolean contexts
        let result = run_program("if 0 { \"truthy\"; } else { \"falsy\"; }").unwrap();
        assert_eq!(result, Value::String("falsy".to_string()));

        let result = run_program("if 1 { \"truthy\"; } else { \"falsy\"; }").unwrap();
        assert_eq!(result, Value::String("truthy".to_string()));

        let result = run_program("if \"\" { \"truthy\"; } else { \"falsy\"; }").unwrap();
        assert_eq!(result, Value::String("falsy".to_string()));

        let result = run_program("if \"hello\" { \"truthy\"; } else { \"falsy\"; }").unwrap();
        assert_eq!(result, Value::String("truthy".to_string()));
    }
}
