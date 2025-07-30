use my::{CompilerError, Interpreter, Lexer, Parser, Value};
use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            start_repl();
        }
        2 => {
            match args[1].as_str() {
                "--help" | "-h" => {
                    // 显示用法信息
                    print_usage(&args[0]);
                }
                file => {
                    // 解释执行文件
                    interpret_file(file);
                }
            }
        }
        _ => {
            // 参数过多时显示错误并退出
            eprintln!("错误：参数过多\n");
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

fn interpret_file(filename: &str) {
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(filename) {
        Ok(source) => match run_interpreter(&source, &mut interpreter) {
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

fn run_interpreter(source: &str, interpreter: &mut Interpreter) -> Result<Value, CompilerError> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    interpreter.interpret(&program)
}

fn print_usage(program_name: &str) {
    println!("Usage:");
    println!("  {} <file>              - Compile file", program_name);
    println!("  {} -                   - Start REPL", program_name);
}

fn start_repl() {
    println!("Interactive Interpreter - Type 'exit' to quit");
    let mut interpreter = Interpreter::new();
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

        match run_interpreter(input, &mut interpreter) {
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
