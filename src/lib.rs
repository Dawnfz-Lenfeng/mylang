pub mod compliler;
pub mod error;
pub mod lexer;
pub mod location;
pub mod parser;
pub mod treewalk;
pub mod vm;

use compliler::Compiler;
use error::{Error, Result};
use lexer::Lexer;
use parser::Parser;
use treewalk::Interpreter;
use vm::VM;

use std::fs;
use std::io::{self, Write};

pub fn run_file_with_tr(filename: &str) {
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(filename) {
        Ok(source) => match run_with_tr(source.to_string(), &mut interpreter) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{}", error.in_file(filename.to_string()));
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("{}", Error::from(error));
            std::process::exit(1);
        }
    }
}

pub fn run_file_with_vm(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(source) => match run_with_vm(source) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{}", error.in_file(filename.to_string()));
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("{}", Error::from(error));
            std::process::exit(1);
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

        match run_with_tr(input.to_string(), &mut interpreter) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{}", error.in_file("<stdin>".to_string()));
            }
        }
    }

    println!("Goodbye!");
}

pub fn print_usage(program_name: &str) {
    println!(
        "Mylang interpreter
    Usage: {program_name} [SCRIPT] [OPTIONS]
    
    Options:
      --tr      Use tree-walk interpreter
      --vm      Use bytecode VM (default)
      --help    Display help information
    
    When no SCRIPT is provided, runs in interactive mode."
    );
}

pub fn run_with_tr(source: String, interpreter: &mut Interpreter) -> Result<()> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let stmts = parser
        .parse()?
        .into_iter()
        .map(|stmt| stmt.into_inner())
        .collect::<Vec<_>>();

    interpreter.interpret(&stmts)?;
    Ok(())
}

/// Run with bytecode VM (alternative execution method)
pub fn run_with_vm(source: String) -> Result<()> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;

    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts)?;

    let mut vm = VM::new(chunk);
    vm.run()?;

    Ok(())
}
