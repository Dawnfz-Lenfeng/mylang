use mylang::lexer::{Lexer, Token, TokenType};

#[cfg(test)]
mod lexer_tests {
    use super::*;

    fn get_tokens(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input.to_string());
        lexer.tokenize().unwrap()
    }

    fn token_types(tokens: &[Token]) -> Vec<TokenType> {
        tokens.iter().map(|t| t.token_type.clone()).collect()
    }

    #[test]
    fn test_empty_input() {
        let tokens = get_tokens("");
        let expected_types = vec![TokenType::Eof];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_whitespace_handling() {
        let tokens = get_tokens("   \t\n   ");
        let expected_types = vec![TokenType::Eof];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_single_character_tokens() {
        let input = "(){}[];,=+-*/<>!";
        let tokens = get_tokens(input);
        let expected_types = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::LeftBracket,
            TokenType::RightBracket,
            TokenType::Semicolon,
            TokenType::Comma,
            TokenType::Equal,
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Star,
            TokenType::Slash,
            TokenType::LessThan,
            TokenType::GreaterThan,
            TokenType::Bang,
            TokenType::Eof,
        ];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_two_character_tokens() {
        let input = "== != <= >= += -= *= /=";
        let tokens = get_tokens(input);

        let expected_types = vec![
            TokenType::EqualEqual,
            TokenType::BangEqual,
            TokenType::LessEqual,
            TokenType::GreaterEqual,
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::StarEqual,
            TokenType::SlashEqual,
            TokenType::Eof,
        ];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_keywords() {
        let input = "let fn if else while for return true false and or";
        let tokens = get_tokens(input);

        let expected_types = vec![
            TokenType::Let,
            TokenType::Fn,
            TokenType::If,
            TokenType::Else,
            TokenType::While,
            TokenType::For,
            TokenType::Return,
            TokenType::Boolean(true),
            TokenType::Boolean(false),
            TokenType::And,
            TokenType::Or,
            TokenType::Eof,
        ];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_identifiers() {
        let input = "variable_name camelCase _underscore var123";
        let tokens = get_tokens(input);

        let expected_types = vec![
            TokenType::Identifier("variable_name".to_string()),
            TokenType::Identifier("camelCase".to_string()),
            TokenType::Identifier("_underscore".to_string()),
            TokenType::Identifier("var123".to_string()),
            TokenType::Eof,
        ];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_numbers() {
        let input = "123 3.14 0 999.999";
        let tokens = get_tokens(input);

        let expected_types = vec![
            TokenType::Number(123.0),
            TokenType::Number(3.14),
            TokenType::Number(0.0),
            TokenType::Number(999.999),
            TokenType::Eof,
        ];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_strings() {
        let input = r#""hello" "world with spaces" 'one "double" quote'"#;
        let tokens = get_tokens(input);

        let expected_types = vec![
            TokenType::String("hello".to_string()),
            TokenType::String("world with spaces".to_string()),
            TokenType::String(r#"one "double" quote"#.to_string()),
            TokenType::Eof,
        ];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_comments() {
        let input = "// this is a comment\nlet x = 42; // another comment";
        let tokens = get_tokens(input);

        // Comments should be ignored, only actual tokens should remain
        let expected_types = vec![
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Equal,
            TokenType::Number(42.0),
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        assert_eq!(token_types(&tokens), expected_types);
    }

    #[test]
    fn test_position_tracking() {
        let input = "let\nx = 42;";
        let tokens = get_tokens(input);

        assert!(tokens.len() >= 2);

        // First token should be on line 1
        assert_eq!(tokens[0].location.line, 1);

        // Check that we have an identifier token on line 2
        if let Some(x_token) = tokens
            .iter()
            .find(|t| matches!(t.token_type, TokenType::Identifier(_)))
        {
            assert_eq!(x_token.location.line, 2);
        }
    }

    #[test]
    fn test_error_handling() {
        let test_cases = vec![
            ("let x = @invalid", "unexpected character: @"),
            ("let x = #hash", "unexpected character: #"),
            ("let x = $dollar", "unexpected character: $"),
            ("let x = `backtick", "unexpected character: `"),
            ("let x = ~tilde", "unexpected character: ~"),
        ];

        for (input, expected_error_part) in test_cases {
            let mut lexer = Lexer::new(input.to_string());
            let result = lexer.tokenize();

            assert!(
                result.is_err(),
                "Expected lexing to fail for input: '{}', but it succeeded with tokens: {:?}",
                input,
                result
            );

            let error = result.unwrap_err();
            assert!(
                error.message.contains(expected_error_part),
                "Error message '{}' should contain '{}'",
                error.message,
                expected_error_part
            );
        }
    }

    #[test]
    fn test_unterminated_string_errors() {
        let test_cases = vec![
            (r#""unterminated"#, "unterminated string literal"),
            (r#"'unterminated"#, "unterminated string literal"),
        ];

        for (input, expected_error_part) in test_cases {
            let mut lexer = Lexer::new(input.to_string());
            let result = lexer.tokenize();

            assert!(
                result.is_err(),
                "Expected lexing to fail for input: '{}', but it succeeded",
                input
            );

            let error = result.unwrap_err();
            assert!(
                error.message.contains(expected_error_part),
                "Error message '{}' should contain '{}'",
                error.message,
                expected_error_part
            );
        }
    }

    #[test]
    fn test_invalid_number_errors() {
        let test_cases = vec![
            ("123.45.67", "unexpected character"),
            ("123..45", "unexpected character"),
            ("1.2.3.4", "unexpected character"),
        ];

        for (input, expected_error_part) in test_cases {
            let mut lexer = Lexer::new(input.to_string());
            let result = lexer.tokenize();

            assert!(
                result.is_err(),
                "Expected lexing to fail for input: '{}', but it succeeded",
                input
            );

            let error = result.unwrap_err();
            assert!(
                error.message.contains(expected_error_part),
                "Error message '{}' should contain '{}'",
                error.message,
                expected_error_part
            );
        }
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let test_cases = vec![
            ("let x = ü", "unexpected character: ü"),
            ("let x = 中文", "unexpected character: 中"),
            ("let x = 🚀", "unexpected character: 🚀"),
            ("let x = €", "unexpected character: €"),
        ];

        for (input, expected_error_part) in test_cases {
            let mut lexer = Lexer::new(input.to_string());
            let result = lexer.tokenize();

            assert!(
                result.is_err(),
                "Expected lexing to fail for input: '{}', but it succeeded",
                input
            );

            let error = result.unwrap_err();
            assert!(
                error.message.contains(expected_error_part),
                "Error message '{}' should contain '{}'",
                error.message,
                expected_error_part
            );
        }
    }

    #[test]
    fn test_error_position_tracking() {
        let input = "let x = @";
        let mut lexer = Lexer::new(input.to_string());
        let result = lexer.tokenize();

        assert!(result.is_err());
        let error = result.unwrap_err();

        // Error should be at line 1, column 9 (location of @)
        assert_eq!(error.location.unwrap().line, 1, "Error line should be 1");
        assert_eq!(
            error.location.unwrap().column,
            9,
            "Error column should be 9"
        );
    }

    #[test]
    fn test_multiline_error_position() {
        let input = "let x = 42;\nlet y = #invalid";
        let mut lexer = Lexer::new(input.to_string());
        let result = lexer.tokenize();

        assert!(result.is_err());
        let error = result.unwrap_err();

        // Error should be at line 2
        assert_eq!(error.location.unwrap().line, 2, "Error line should be 2");
        assert_eq!(
            error.location.unwrap().column,
            9,
            "Error column should be 9"
        );
    }

    #[test]
    fn test_function_definition() {
        let input = "fn add(a, b) { return a + b; }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();

        let expected_types = vec![
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

        assert_eq!(token_types(&tokens), expected_types);
    }
}
