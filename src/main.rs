use my::{Interpreter, Lexer, Parser, Value, CompilerError};
use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:");
        println!("  {} <file>              - Compile file", args[0]);
        println!("  {} --repl              - Start REPL", args[0]);
        return;
    }

    match args[1].as_str() {
        "--repl" => {
            start_repl();
        }
        file => {
            interpret_file(file);
        }
    }
}

fn interpret_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(source) => match run_interpreter(&source) {
            Ok(result) => {
                if !matches!(result, my::Value::Null) {
                    println!("{}", result);
                }
            }
            Err(error) => {
                eprintln!("Runtime Error: {}", error);
            }
        },
        Err(error) => {
            eprintln!("Error reading file '{}': {}", filename, error);
        }
    }
}

fn run_interpreter(source: &str) -> Result<Value, CompilerError> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&program)
}

fn start_repl() {
    println!("Interactive Interpreter - Type 'exit' to quit");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input == "exit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        match run_interpreter(input) {
            Ok(result) => {
                if !matches!(result, Value::Null) {
                    println!("{}", result);
                }
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }

    println!("Goodbye!");
}
