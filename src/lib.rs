pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod utils;

use error::{Error, Result};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

use std::fs;
use std::io::{self, Write};

pub fn run_file(filename: &str) {
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(filename) {
        Ok(source) => match run(source, &mut interpreter) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{error}");
            }
        },
        Err(error) => {
            eprintln!("{}", Error::from(error));
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
            Ok(_) => (),
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

fn run(source: String, interpreter: &mut Interpreter) -> Result<()> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;

    for stmt in stmts {
        stmt.accept(interpreter)?;
    }
    Ok(())
}
