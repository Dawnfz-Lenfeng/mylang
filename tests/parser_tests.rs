use mylang::{
    error::Result,
    lexer::lexer::Lexer,
    parser::{BinaryOp, Expr, Parser, Stmt, UnaryOp},
};

#[cfg(test)]
mod parser_tests {
    use super::*;

    fn parse_program(input: &str) -> Result<Vec<Stmt>> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_number_literal() {
        let program = parse_program("42;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Number(42.0))];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_string_literal() {
        let program = parse_program(r#""hello";"#).unwrap();
        let expected = vec![Stmt::Expression(Expr::String("hello".to_string()))];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_boolean_literals() {
        let program_true = parse_program("true;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Boolean(true))];
        assert_eq!(program_true, expected);

        let program_false = parse_program("false;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Boolean(false))];
        assert_eq!(program_false, expected);
    }

    #[test]
    fn test_identifier() {
        let program = parse_program("variable;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Variable("variable".to_string()))];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_binary_expressions() {
        // Test addition
        let program = parse_program("1 + 2;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Binary {
            left: Box::new(Expr::Number(1.0)),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Number(2.0)),
        })];
        assert_eq!(program, expected);

        // Test multiplication
        let program = parse_program("3 * 4;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Binary {
            left: Box::new(Expr::Number(3.0)),
            operator: BinaryOp::Multiply,
            right: Box::new(Expr::Number(4.0)),
        })];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_operator_precedence_structure() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let program = parse_program("1 + 2 * 3;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Binary {
            left: Box::new(Expr::Number(1.0)),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(2.0)),
                operator: BinaryOp::Multiply,
                right: Box::new(Expr::Number(3.0)),
            }),
        })];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_comparison_precedence() {
        // a < b and c > d should parse as (a < b) and (c > d)
        let program = parse_program("a < b and c > d;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Variable("a".to_string())),
                operator: BinaryOp::LessThan,
                right: Box::new(Expr::Variable("b".to_string())),
            }),
            operator: BinaryOp::LogicalAnd,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Variable("c".to_string())),
                operator: BinaryOp::GreaterThan,
                right: Box::new(Expr::Variable("d".to_string())),
            }),
        })];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_logical_expressions() {
        let test_cases = vec![
            ("a and b;", "a", "b", BinaryOp::LogicalAnd),
            ("x or y;", "x", "y", BinaryOp::LogicalOr),
        ];

        for (input, left_name, right_name, expected_op) in test_cases {
            let program = parse_program(input).unwrap();
            let expected = vec![Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Variable(left_name.to_string())),
                operator: expected_op,
                right: Box::new(Expr::Variable(right_name.to_string())),
            })];
            assert_eq!(program, expected);
        }
    }

    #[test]
    fn test_unary_expressions() {
        // Test unary minus
        let program = parse_program("-42;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Unary {
            operator: UnaryOp::Minus,
            operand: Box::new(Expr::Number(42.0)),
        })];
        assert_eq!(program, expected);

        // Test unary not
        let program = parse_program("!true;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Unary {
            operator: UnaryOp::Not,
            operand: Box::new(Expr::Boolean(true)),
        })];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_parenthesized_expressions() {
        let program = parse_program("(1 + 2) * 3;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(1.0)),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Number(2.0)),
            }),
            operator: BinaryOp::Multiply,
            right: Box::new(Expr::Number(3.0)),
        })];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_comparison_expressions() {
        let test_cases = vec![
            ("a == b;", "a", "b", BinaryOp::Equal),
            ("x != y;", "x", "y", BinaryOp::NotEqual),
            ("a < b;", "a", "b", BinaryOp::LessThan),
            ("x <= y;", "x", "y", BinaryOp::LessEqual),
            ("a > b;", "a", "b", BinaryOp::GreaterThan),
            ("x >= y;", "x", "y", BinaryOp::GreaterEqual),
        ];

        for (input, left_name, right_name, expected_op) in test_cases {
            let program = parse_program(input).unwrap();
            let expected = vec![Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Variable(left_name.to_string())),
                operator: expected_op,
                right: Box::new(Expr::Variable(right_name.to_string())),
            })];
            assert_eq!(program, expected);
        }
    }

    #[test]
    fn test_function_call_structure() {
        let program = parse_program("func(1, 2, x);").unwrap();
        let expected = vec![Stmt::Expression(Expr::Call {
            callee: Box::new(Expr::Variable("func".to_string())),
            arguments: vec![
                Expr::Number(1.0),
                Expr::Number(2.0),
                Expr::Variable("x".to_string()),
            ],
        })];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_let_statement_structure() {
        let program = parse_program("let x = 42;").unwrap();
        let expected = vec![Stmt::VarDecl {
            name: "x".to_string(),
            initializer: Some(Expr::Number(42.0)),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_function_declaration_structure() {
        let program = parse_program("fn add(a, b) { return a + b; }").unwrap();
        let expected = vec![Stmt::FuncDecl {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: Box::new(Stmt::Block(vec![Stmt::Return {
                value: Some(Expr::Binary {
                    left: Box::new(Expr::Variable("a".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Variable("b".to_string())),
                }),
            }])),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_if_statement_structure() {
        let program = parse_program("if x > 0 { x; } else { y; }").unwrap();
        let expected = vec![Stmt::If {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable("x".to_string())),
                operator: BinaryOp::GreaterThan,
                right: Box::new(Expr::Number(0.0)),
            },
            then_branch: Box::new(Stmt::Block(vec![Stmt::Expression(Expr::Variable(
                "x".to_string(),
            ))])),
            else_branch: Some(Box::new(Stmt::Block(vec![Stmt::Expression(
                Expr::Variable("y".to_string()),
            )]))),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_while_statement_structure() {
        let program = parse_program("while x > 0 { x = x - 1; }").unwrap();
        let expected = vec![Stmt::While {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable("x".to_string())),
                operator: BinaryOp::GreaterThan,
                right: Box::new(Expr::Number(0.0)),
            },
            body: Box::new(Stmt::Block(vec![Stmt::Expression(Expr::Assign {
                name: "x".to_string(),
                value: Box::new(Expr::Binary {
                    left: Box::new(Expr::Variable("x".to_string())),
                    operator: BinaryOp::Subtract,
                    right: Box::new(Expr::Number(1.0)),
                }),
            })])),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_operator_precedence() {
        let test_cases = vec![
            (
                "1 + 2 * 3;",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Number(1.0)),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Number(2.0)),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expr::Number(3.0)),
                    }),
                })],
            ),
            (
                "a and b or c;",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("a".to_string())),
                        operator: BinaryOp::LogicalAnd,
                        right: Box::new(Expr::Variable("b".to_string())),
                    }),
                    operator: BinaryOp::LogicalOr,
                    right: Box::new(Expr::Variable("c".to_string())),
                })],
            ), // and before or
            (
                "!a and b;",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Unary {
                        operator: UnaryOp::Not,
                        operand: Box::new(Expr::Variable("a".to_string())),
                    }),
                    operator: BinaryOp::LogicalAnd,
                    right: Box::new(Expr::Variable("b".to_string())),
                })],
            ), // not before and
            (
                "a < b and c > d;",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("a".to_string())),
                        operator: BinaryOp::LessThan,
                        right: Box::new(Expr::Variable("b".to_string())),
                    }),
                    operator: BinaryOp::LogicalAnd,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("c".to_string())),
                        operator: BinaryOp::GreaterThan,
                        right: Box::new(Expr::Variable("d".to_string())),
                    }),
                })],
            ), // comparison before logical
        ];

        for (input, expected) in test_cases {
            let result = parse_program(input).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_complex_expressions() {
        let test_cases = vec![
            (
                "a + b * c - d / e;",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Binary {
                            left: Box::new(Expr::Variable("b".to_string())),
                            operator: BinaryOp::Multiply,
                            right: Box::new(Expr::Variable("c".to_string())),
                        }),
                    }),
                    operator: BinaryOp::Subtract,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("d".to_string())),
                        operator: BinaryOp::Divide,
                        right: Box::new(Expr::Variable("e".to_string())),
                    }),
                })],
            ),
            (
                "func(a + b, c * d);",
                vec![Stmt::Expression(Expr::Call {
                    callee: Box::new(Expr::Variable("func".to_string())),
                    arguments: vec![
                        Expr::Binary {
                            left: Box::new(Expr::Variable("a".to_string())),
                            operator: BinaryOp::Add,
                            right: Box::new(Expr::Variable("b".to_string())),
                        },
                        Expr::Binary {
                            left: Box::new(Expr::Variable("c".to_string())),
                            operator: BinaryOp::Multiply,
                            right: Box::new(Expr::Variable("d".to_string())),
                        },
                    ],
                })],
            ),
            (
                "(a + b) * (c - d);",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Variable("b".to_string())),
                    }),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("c".to_string())),
                        operator: BinaryOp::Subtract,
                        right: Box::new(Expr::Variable("d".to_string())),
                    }),
                })],
            ),
            (
                "!flag and count > 0;",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Unary {
                        operator: UnaryOp::Not,
                        operand: Box::new(Expr::Variable("flag".to_string())),
                    }),
                    operator: BinaryOp::LogicalAnd,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("count".to_string())),
                        operator: BinaryOp::GreaterThan,
                        right: Box::new(Expr::Number(0.0)),
                    }),
                })],
            ),
            (
                "x == y or z != w and a < b;",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("x".to_string())),
                        operator: BinaryOp::Equal,
                        right: Box::new(Expr::Variable("y".to_string())),
                    }),
                    operator: BinaryOp::LogicalOr,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Binary {
                            left: Box::new(Expr::Variable("z".to_string())),
                            operator: BinaryOp::NotEqual,
                            right: Box::new(Expr::Variable("w".to_string())),
                        }),
                        operator: BinaryOp::LogicalAnd,
                        right: Box::new(Expr::Binary {
                            left: Box::new(Expr::Variable("a".to_string())),
                            operator: BinaryOp::LessThan,
                            right: Box::new(Expr::Variable("b".to_string())),
                        }),
                    }),
                })],
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_program(input).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_function_call() {
        let result = parse_program("func(1, 2, x);").unwrap();
        let expected = vec![Stmt::Expression(Expr::Call {
            callee: Box::new(Expr::Variable("func".to_string())),
            arguments: vec![
                Expr::Number(1.0),
                Expr::Number(2.0),
                Expr::Variable("x".to_string()),
            ],
        })];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_function_call_no_args() {
        let program = parse_program("func();").unwrap();
        let expected = vec![Stmt::Expression(Expr::Call {
            callee: Box::new(Expr::Variable("func".to_string())),
            arguments: vec![],
        })];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_let_statement() {
        let result = parse_program("let x = 42;").unwrap();
        let expected = vec![Stmt::VarDecl {
            name: "x".to_string(),
            initializer: Some(Expr::Number(42.0)),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_assignment_statement() {
        let result = parse_program("x = 42;").unwrap();
        let expected = vec![Stmt::Expression(Expr::Assign {
            name: "x".to_string(),
            value: Box::new(Expr::Number(42.0)),
        })];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_assignment_expressions() {
        let test_cases = vec![
            (
                "x = y = z;",
                vec![Stmt::Expression(Expr::Assign {
                    name: "x".to_string(),
                    value: Box::new(Expr::Assign {
                        name: "y".to_string(),
                        value: Box::new(Expr::Variable("z".to_string())),
                    }),
                })],
            ),
            (
                "a = b + c;",
                vec![Stmt::Expression(Expr::Assign {
                    name: "a".to_string(),
                    value: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("b".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Variable("c".to_string())),
                    }),
                })],
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_program(input).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_if_statement() {
        let result = parse_program("if x > 0 { x; }").unwrap();
        let expected = vec![Stmt::If {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable("x".to_string())),
                operator: BinaryOp::GreaterThan,
                right: Box::new(Expr::Number(0.0)),
            },
            then_branch: Box::new(Stmt::Block(vec![Stmt::Expression(Expr::Variable(
                "x".to_string(),
            ))])),
            else_branch: None,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_if_else_statement() {
        let result = parse_program("if x > 0 { x; } else { y; }").unwrap();
        let expected = vec![Stmt::If {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable("x".to_string())),
                operator: BinaryOp::GreaterThan,
                right: Box::new(Expr::Number(0.0)),
            },
            then_branch: Box::new(Stmt::Block(vec![Stmt::Expression(Expr::Variable(
                "x".to_string(),
            ))])),
            else_branch: Some(Box::new(Stmt::Block(vec![Stmt::Expression(
                Expr::Variable("y".to_string()),
            )]))),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_while_statement() {
        let result = parse_program("while x > 0 { x = x - 1; }").unwrap();
        let expected = vec![Stmt::While {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable("x".to_string())),
                operator: BinaryOp::GreaterThan,
                right: Box::new(Expr::Number(0.0)),
            },
            body: Box::new(Stmt::Block(vec![Stmt::Expression(Expr::Assign {
                name: "x".to_string(),
                value: Box::new(Expr::Binary {
                    left: Box::new(Expr::Variable("x".to_string())),
                    operator: BinaryOp::Subtract,
                    right: Box::new(Expr::Number(1.0)),
                }),
            })])),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_for_statement() {
        let result = parse_program(
            "for let i = 0; i < 10; i = i + 1 { 
                print i; 
            }",
        )
        .unwrap();

        let expected = vec![Stmt::Block(vec![
            Stmt::VarDecl {
                name: "i".to_string(),
                initializer: Some(Expr::Number(0.0)),
            },
            Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Variable("i".to_string())),
                    operator: BinaryOp::LessThan,
                    right: Box::new(Expr::Number(10.0)),
                },
                body: Box::new(Stmt::Block(vec![
                    Stmt::Block(vec![Stmt::Print(vec![Expr::Variable("i".to_string())])]),
                    Stmt::Expression(Expr::Assign {
                        name: "i".to_string(),
                        value: Box::new(Expr::Binary {
                            left: Box::new(Expr::Variable("i".to_string())),
                            operator: BinaryOp::Add,
                            right: Box::new(Expr::Number(1.0)),
                        }),
                    }),
                ])),
            },
        ])];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_function_declaration() {
        let result = parse_program("fn add(a, b) { return a + b; }").unwrap();
        let expected = vec![Stmt::FuncDecl {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: Box::new(Stmt::Block(vec![Stmt::Return {
                value: Some(Expr::Binary {
                    left: Box::new(Expr::Variable("a".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Variable("b".to_string())),
                }),
            }])),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_function_declaration_no_params() {
        let result = parse_program("fn hello() { return \"world\"; }").unwrap();
        let expected = vec![Stmt::FuncDecl {
            name: "hello".to_string(),
            params: vec![],
            body: Box::new(Stmt::Block(vec![Stmt::Return {
                value: Some(Expr::String("world".to_string())),
            }])),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_return_statement() {
        let result = parse_program("return 42;").unwrap();
        let expected = vec![Stmt::Return {
            value: Some(Expr::Number(42.0)),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_return_statement_no_value() {
        let program = parse_program("return;").unwrap();
        let expected = vec![Stmt::Return { value: None }];
        assert_eq!(program, expected);
    }

    #[test]
    fn test_block_statement() {
        let result = parse_program("{ let x = 1; x; }").unwrap();
        let expected = vec![Stmt::Block(vec![
            Stmt::VarDecl {
                name: "x".to_string(),
                initializer: Some(Expr::Number(1.0)),
            },
            Stmt::Expression(Expr::Variable("x".to_string())),
        ])];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_blocks() {
        let input = r#"
            {
                let x = 1;
                {
                    let y = 2;
                    x + y;
                }
            }
        "#;
        let result = parse_program(input).unwrap();
        let expected = vec![Stmt::Block(vec![
            Stmt::VarDecl {
                name: "x".to_string(),
                initializer: Some(Expr::Number(1.0)),
            },
            Stmt::Block(vec![
                Stmt::VarDecl {
                    name: "y".to_string(),
                    initializer: Some(Expr::Number(2.0)),
                },
                Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Variable("x".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Variable("y".to_string())),
                }),
            ]),
        ])];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_complex_program() {
        let input = r#"
            fn main() {
                let x = 42;
                let y = x + 1;
                if y > x {
                    return y;
                }
                return x;
            }
        "#;

        let result = parse_program(input);
        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_fibonacci_example() {
        let input = r#"
            fn fibonacci(n) {
                if n <= 1 {
                    return n;
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }

            fn main() {
                let result = fibonacci(10);
                return result;
            }
        "#;

        let result = parse_program(input);
        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn test_error_recovery() {
        let result = parse_program("let x = 42");
        assert!(result.is_err());

        let result = parse_program("1 + + 2;");
        assert!(result.is_err());
    }

    #[test]
    fn test_syntax_errors() {
        let error_cases = vec![
            "let = 42;",      // missing identifier
            "fn () { }",      // missing function name
            "if { }",         // missing condition
            "while { }",      // missing condition
            "return return;", // invalid return value
        ];

        for input in error_cases {
            let result = parse_program(input);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_nested_expressions() {
        let result = parse_program("func(a + b, c * d);").unwrap();
        let expected = vec![Stmt::Expression(Expr::Call {
            callee: Box::new(Expr::Variable("func".to_string())),
            arguments: vec![
                Expr::Binary {
                    left: Box::new(Expr::Variable("a".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Variable("b".to_string())),
                },
                Expr::Binary {
                    left: Box::new(Expr::Variable("c".to_string())),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expr::Variable("d".to_string())),
                },
            ],
        })];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deeply_nested_expressions() {
        let result = parse_program("func(func2(a + b), func3(c * func4(d)));").unwrap();
        let expected = vec![Stmt::Expression(Expr::Call {
            callee: Box::new(Expr::Variable("func".to_string())),
            arguments: vec![
                Expr::Call {
                    callee: Box::new(Expr::Variable("func2".to_string())),
                    arguments: vec![Expr::Binary {
                        left: Box::new(Expr::Variable("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Variable("b".to_string())),
                    }],
                },
                Expr::Call {
                    callee: Box::new(Expr::Variable("func3".to_string())),
                    arguments: vec![Expr::Binary {
                        left: Box::new(Expr::Variable("c".to_string())),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expr::Call {
                            callee: Box::new(Expr::Variable("func4".to_string())),
                            arguments: vec![Expr::Variable("d".to_string())],
                        }),
                    }],
                },
            ],
        })];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_program() {
        let program = parse_program("").unwrap();
        assert_eq!(program.len(), 0);
    }

    #[test]
    fn test_whitespace_and_comments() {
        let input = r#"
            // This is a comment
            let x = 42; // Another comment
            
            fn test() {
                // Function comment
                return x;
            }
        "#;
        let result = parse_program(input).unwrap();
        let expected = vec![
            Stmt::VarDecl {
                name: "x".to_string(),
                initializer: Some(Expr::Number(42.0)),
            },
            Stmt::FuncDecl {
                name: "test".to_string(),
                params: vec![],
                body: Box::new(Stmt::Block(vec![Stmt::Return {
                    value: Some(Expr::Variable("x".to_string())),
                }])),
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_statements() {
        let input = r#"
            let x = 1;
            let y = 2;
            let z = x + y;
        "#;

        let result = parse_program(input);
        assert!(result.is_ok());

        let program = result.unwrap();
        assert!(program.len() >= 3);
    }

    #[test]
    fn test_expression_statement_precedence() {
        // Test that assignment has lower precedence than other operations
        let input = "x = a + b * c;";
        let result = parse_program(input).unwrap();
        let expected = vec![Stmt::Expression(Expr::Assign {
            name: "x".to_string(),
            value: Box::new(Expr::Binary {
                left: Box::new(Expr::Variable("a".to_string())),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Variable("b".to_string())),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expr::Variable("c".to_string())),
                }),
            }),
        })];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_chained_comparisons() {
        // Note: This might not be supported depending on grammar design
        let input = "a < b and b < c;";
        let result = parse_program(input).unwrap();
        let expected = vec![Stmt::Expression(Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Variable("a".to_string())),
                operator: BinaryOp::LessThan,
                right: Box::new(Expr::Variable("b".to_string())),
            }),
            operator: BinaryOp::LogicalAnd,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Variable("b".to_string())),
                operator: BinaryOp::LessThan,
                right: Box::new(Expr::Variable("c".to_string())),
            }),
        })];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_function_call_in_expressions() {
        let test_cases = vec![
            (
                "x + func(y);",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Variable("x".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Call {
                        callee: Box::new(Expr::Variable("func".to_string())),
                        arguments: vec![Expr::Variable("y".to_string())],
                    }),
                })],
            ),
            (
                "func1(func2(x));",
                vec![Stmt::Expression(Expr::Call {
                    callee: Box::new(Expr::Variable("func1".to_string())),
                    arguments: vec![Expr::Call {
                        callee: Box::new(Expr::Variable("func2".to_string())),
                        arguments: vec![Expr::Variable("x".to_string())],
                    }],
                })],
            ),
            (
                "func(x) + func(y);",
                vec![Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Call {
                        callee: Box::new(Expr::Variable("func".to_string())),
                        arguments: vec![Expr::Variable("x".to_string())],
                    }),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Call {
                        callee: Box::new(Expr::Variable("func".to_string())),
                        arguments: vec![Expr::Variable("y".to_string())],
                    }),
                })],
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_program(input).unwrap();
            assert_eq!(result, expected);
        }
    }
}
