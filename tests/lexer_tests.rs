use interpreter::{Lexer, Token, TokenType, CompilerError};

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.is_empty() || tokens.last().unwrap().token_type == TokenType::Eof);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("   \t\n   ".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.is_empty() || tokens.last().unwrap().token_type == TokenType::Eof);
    }

    #[test]
    fn test_single_character_tokens() {
        let input = "(){}[];,+-*/=<>!";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::LeftBracket,
            TokenType::RightBracket,
            TokenType::Semicolon,
            TokenType::Comma,
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Multiply,
            TokenType::Divide,
            TokenType::Assign,
            TokenType::LessThan,
            TokenType::GreaterThan,
            TokenType::Not,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected.len(),
            "Single char tokens: Token count mismatch!\nExpected: {} tokens\nActual: {} tokens\nTokens: {:#?}", 
            expected.len(), tokens.len(), tokens);
        for (i, expected_token) in expected.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&tokens[i].token_type),
                std::mem::discriminant(expected_token),
                "Single char tokens: Token type mismatch at position {}!\nActual token: {:#?}\nExpected type: {:#?}",
                i, tokens[i], expected_token
            );
        }
    }

    #[test]
    fn test_two_character_tokens() {
        let input = "== != <= >= && ||";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected = vec![
            TokenType::Equal,
            TokenType::NotEqual,
            TokenType::LessEqual,
            TokenType::GreaterEqual,
            TokenType::And,
            TokenType::Or,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected.len(),
            "Two char tokens: Token count mismatch!\nExpected: {} tokens\nActual: {} tokens\nTokens: {:#?}", 
            expected.len(), tokens.len(), tokens);
        for (i, expected_token) in expected.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&tokens[i].token_type),
                std::mem::discriminant(expected_token),
                "Two char tokens: Token type mismatch at position {}!\nActual token: {:#?}\nExpected type: {:#?}",
                i, tokens[i], expected_token
            );
        }
    }

    #[test]
    fn test_keywords() {
        let input = "let fn if else while for return true false";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected = vec![
            TokenType::Let,
            TokenType::Fn,
            TokenType::If,
            TokenType::Else,
            TokenType::While,
            TokenType::For,
            TokenType::Return,
            TokenType::True,
            TokenType::False,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected.len(), 
            "Token count mismatch!\nExpected: {} tokens\nActual: {} tokens\nTokens: {:#?}", 
            expected.len(), tokens.len(), tokens);
            
        for (i, expected_token) in expected.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&tokens[i].token_type),
                std::mem::discriminant(expected_token),
                "Token type mismatch at position {}!\nActual token: {:#?}\nExpected type: {:#?}\nAll tokens: {:#?}",
                i, tokens[i], expected_token, tokens
            );
        }
    }

    #[test]
    fn test_identifiers() {
        let input = "variable_name camelCase _underscore var123";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected_names = vec!["variable_name", "camelCase", "_underscore", "var123"];
        let identifier_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::Identifier(_)))
            .collect();

        assert_eq!(identifier_tokens.len(), expected_names.len(),
            "Identifier tokens: Count mismatch!\nExpected: {} identifiers\nActual: {} identifiers\nAll tokens: {:#?}", 
            expected_names.len(), identifier_tokens.len(), tokens);
        for (i, expected_name) in expected_names.iter().enumerate() {
            if let TokenType::Identifier(name) = &identifier_tokens[i].token_type {
                assert_eq!(name, expected_name,
                    "Identifier mismatch at position {}!\nActual: '{}'\nExpected: '{}'",
                    i, name, expected_name);
            } else {
                panic!("Expected identifier token at position {}, got: {:#?}", i, identifier_tokens[i]);
            }
        }
    }

    #[test]
    fn test_numbers() {
        let input = "123 3.14 0 999.999";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected_numbers = vec![123.0, 3.14, 0.0, 999.999];
        let number_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::Number(_)))
            .collect();

        assert_eq!(number_tokens.len(), expected_numbers.len(),
            "Number tokens: Count mismatch!\nExpected: {} numbers\nActual: {} numbers\nAll tokens: {:#?}", 
            expected_numbers.len(), number_tokens.len(), tokens);
        for (i, expected_num) in expected_numbers.iter().enumerate() {
            if let TokenType::Number(num) = number_tokens[i].token_type {
                assert_eq!(num, *expected_num,
                    "Number mismatch at position {}!\nActual: {}\nExpected: {}",
                    i, num, expected_num);
            } else {
                panic!("Expected number token at position {}, got: {:#?}", i, number_tokens[i]);
            }
        }
    }

    #[test]
    fn test_strings() {
        let input = r#""hello" "world with spaces" "escaped\"quote""#;
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected_strings = vec!["hello", "world with spaces", "escaped\"quote"];
        let string_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::String(_)))
            .collect();

        assert_eq!(string_tokens.len(), expected_strings.len(),
            "String tokens: Count mismatch!\nExpected: {} strings\nActual: {} strings\nAll tokens: {:#?}", 
            expected_strings.len(), string_tokens.len(), tokens);
        for (i, expected_str) in expected_strings.iter().enumerate() {
            if let TokenType::String(s) = &string_tokens[i].token_type {
                assert_eq!(s, expected_str,
                    "String mismatch at position {}!\nActual: '{}'\nExpected: '{}'",
                    i, s, expected_str);
            } else {
                panic!("Expected string token at position {}, got: {:#?}", i, string_tokens[i]);
            }
        }
    }

    #[test]
    fn test_comments() {
        let input = "// this is a comment\nlet x = 42; // another comment";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        // Comments should be ignored, only actual tokens should remain
        let expected_types = vec![
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::Number(42.0),
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len(),
            "Comment test: Token count mismatch!\nExpected: {} tokens\nActual: {} tokens\nTokens: {:#?}", 
            expected_types.len(), tokens.len(), tokens);
        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&tokens[i].token_type),
                std::mem::discriminant(expected_type),
                "Comment test: Token type mismatch at position {}!\nActual token: {:#?}\nExpected type: {:#?}",
                i, tokens[i], expected_type
            );
        }
    }

    #[test]
    fn test_position_tracking() {
        let input = "let\nx = 42;";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(tokens.len() >= 2);

        // First token should be on line 1
        assert_eq!(tokens[0].line, 1);

        // Check that we have an identifier token on line 2
        if let Some(x_token) = tokens
            .iter()
            .find(|t| matches!(t.token_type, TokenType::Identifier(_)))
        {
            assert_eq!(x_token.line, 2);
        }
    }

    #[test]
    fn test_error_handling() {
        let input = "let x = @invalid";
        let mut lexer = Lexer::new(input.to_string());

        // Should produce an error for invalid character
        let result = lexer.tokenize();
        assert!(
            result.is_err(),
            "Expected lexing to fail with invalid character"
        );
    }

    #[test]
    fn test_function_definition() {
        let input = "fn add(a, b) { return a + b; }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected = vec![
            TokenType::Fn,
            TokenType::Identifier("add".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("a".to_string()),
            TokenType::Comma,
            TokenType::Identifier("b".to_string()),
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::Return,
            TokenType::Identifier("a".to_string()),
            TokenType::Plus,
            TokenType::Identifier("b".to_string()),
            TokenType::Semicolon,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected.len(),
            "Function definition: Token count mismatch!\nExpected: {} tokens\nActual: {} tokens\nTokens: {:#?}", 
            expected.len(), tokens.len(), tokens);
        for (i, expected_token) in expected.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&tokens[i].token_type),
                std::mem::discriminant(expected_token),
                "Function definition: Token type mismatch at position {}!\nActual token: {:#?}\nExpected type: {:#?}",
                i, tokens[i], expected_token
            );
        }
    }
}
