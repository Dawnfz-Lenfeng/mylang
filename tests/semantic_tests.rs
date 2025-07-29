use interpreter::{CompilerError, Lexer, Parser, SemanticAnalyzer};

#[cfg(test)]
mod semantic_tests {
    use super::*;

    fn analyze_program(input: &str) -> Result<(), CompilerError> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&program)
    }

    #[test]
    fn test_variable_declaration() {
        let input = r#"
            fn main() {
                let x = 42;
                let y = "hello";
                let z = true;
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_declaration() {
        let input = r#"
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            
            fn main() {
                let result = add(1, 2);
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scope_management() {
        let input = r#"
            fn main() {
                let x = 42;
                if true {
                    let y = x + 1;
                }
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_variable_error() {
        let input = r#"
            fn main() {
                let x = y + 1;
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("Undefined variable"));
        }
    }

    #[test]
    fn test_type_mismatch_error() {
        let input = r#"
            fn main() {
                let x: number = 42;
                let y: str = "hello";
                let z = x + y;
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(
                error
                    .message
                    .contains("Arithmetic operations require numeric operands")
            );
        }
    }

    #[test]
    fn test_function_call_validation() {
        let input = r#"
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            
            fn main() {
                let result1 = add(1, 2);
                let result2 = add(1);
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(
                error
                    .message
                    .contains("Function expects 2 arguments, got 1")
            );
        }
    }

    #[test]
    fn test_control_flow_analysis() {
        let input = r#"
            fn main() {
                let x = 10;
                if x > 5 {
                    let y = x * 2;
                } else {
                    let z = x / 2;
                }
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
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
            
            fn main() {
                let result = factorial(5);
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_boolean_expressions() {
        let input = r#"
            fn main() {
                let a = 5;
                let b = 10;
                let result1 = a < b;
                let result2 = a == b;
                let result3 = result1 and result2;
                let result4 = not result3;
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assignment_type_checking() {
        let input = r#"
            fn main() {
                let x = 42;
                x = 24;
                x = "invalid";
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("Type mismatch"));
        }
    }

    #[test]
    fn test_nested_scopes() {
        let input = r#"
            fn main() {
                let x = 1;
                {
                    let y = 2;
                    {
                        let z = x + y;
                    }
                }
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_parameter_scoping() {
        let input = r#"
            fn test(param: number) -> number {
                let local = param + 1;
                return local;
            }
            
            fn main() {
                let result = test(42);
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_expressions() {
        let input = r#"
            fn main() {
                let a = 1;
                let b = 2;
                let c = 3;
                let result = (a + b) * c - (a * b + c);
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_program() {
        let input = "";
        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_functions() {
        let input = r#"
            fn helper1() -> number {
                return 42;
            }
            
            fn helper2(x: number) -> number {
                return x * 2;
            }
            
            fn main() {
                let value = helper1();
                let doubled = helper2(value);
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_const_immutability() {
        let input = r#"
            fn main() {
                const x = 42;
                x = 24;  // Should fail - cannot modify const
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(
                error
                    .message
                    .contains("Cannot assign to immutable variable")
            );
        }
    }

    #[test]
    fn test_let_mutability() {
        let input = r#"
            fn main() {
                let x = 42;
                x = 24;  // Should succeed - let allows modification
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_type_checking() {
        let input = r#"
            fn main() {
                let arr: array[number] = [1, 2, 3];
                let element = arr[0];
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_index_type_error() {
        let input = r#"
            fn main() {
                let arr = [1, 2, 3];
                let element = arr["invalid"];  // Should fail - index must be number
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("Array index must be numeric"));
        }
    }

    #[test]
    fn test_return_type_mismatch() {
        let input = r#"
            fn test() -> number {
                return "string";  // Should fail - return type mismatch
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("Return type mismatch"));
        }
    }

    #[test]
    fn test_variable_shadowing() {
        let input = r#"
            fn main() {
                let x = 42;
                {
                    let x = "shadow";  // Should be allowed - different scope
                    let y = x;  // y should be string type
                }
                let z = x;  // z should be number type
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uninitialized_variable_usage() {
        let input = r#"
            fn main() {
                let x: number;
                let y = x + 1;  // Should fail - x not initialized
            }
        "#;

        let result = analyze_program(input);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("Use of uninitialized variable"));
        }
    }
}
