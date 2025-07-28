use interpreter::{Compiler, TargetPlatform};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file> [target]", args[0]);
        eprintln!("Targets: bytecode, javascript, llvm, assembly");
        process::exit(1);
    }

    let filename = &args[1];
    let target = if args.len() > 2 {
        match args[2].to_lowercase().as_str() {
            "bytecode" => TargetPlatform::Bytecode,
            "javascript" | "js" => TargetPlatform::JavaScript,
            "llvm" => TargetPlatform::LLVM,
            "assembly" | "asm" => TargetPlatform::Assembly,
            _ => {
                eprintln!("Unknown target: {}", args[2]);
                eprintln!("Valid targets: bytecode, javascript, llvm, assembly");
                process::exit(1);
            }
        }
    } else {
        TargetPlatform::JavaScript // Default target
    };

    let source_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    println!("Compiling file: {}", filename);
    println!("Target platform: {:?}", target);
    println!("{}", "=".repeat(50));

    let mut compiler = Compiler::new();

    match compiler.compile(&source_code, target) {
        Ok(output) => {
            println!("Compilation successful!");
            println!("{}", "=".repeat(50));
            println!("Generated code:");
            println!("{}", output);
        }
        Err(error) => {
            eprintln!("Compilation failed: {}", error);
            compiler.error_reporter().report_all();
            process::exit(1);
        }
    }
}
