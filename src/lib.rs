pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod utils;

use error::Result;
use interpreter::{Interpreter, Value};
use lexer::Lexer;
use parser::Parser;

use std::fs;
use std::io::{self, Write};

pub fn run_file(filename: &str) {
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(filename) {
        Ok(source) => match run(source, &mut interpreter) {
            Ok(result) => {
                if !matches!(result, Value::Null) {
                    println!("{result}");
                }
            }
            Err(error) => {
                eprintln!("{error}");
            }
        },
        Err(error) => {
            eprintln!("Error reading file '{filename}': {error}");
        }
    }
}

pub fn run_prompt() {
    println!("Interactive Interpreter - Type 'exit' to quit");
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("valid user input");

        let input = input.trim();
        if input == "exit" {
            break;
        }

        match run(input.to_string(), &mut interpreter) {
            Ok(result) => {
                if !matches!(result, Value::Null) {
                    println!("{}", result);
                }
            }
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }

    println!("Goodbye!");
}

pub fn print_usage(program_name: &str) {
    println!("Usage: {program_name} [script]");
}

fn run(source: String, interpreter: &mut Interpreter) -> Result<Value> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    interpreter.interpret(&program)
}
