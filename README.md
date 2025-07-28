# Rust Compiler/Interpreter

A simple compiler and interpreter built in Rust, supporting multiple target platforms.

## Project Structure

```
interpreter/
├── src/
│   ├── main.rs          # Entry point & CLI
│   ├── lib.rs           # Library interface
│   ├── lexer.rs         # Lexical analysis (tokenization)
│   ├── parser.rs        # Syntax analysis (parsing)
│   ├── ast.rs           # Abstract Syntax Tree definitions
│   ├── semantic.rs      # Semantic analysis & type checking
│   ├── codegen.rs       # Code generation for multiple targets
│   ├── error.rs         # Error handling & reporting
│   └── utils.rs         # Utility functions & helpers
├── examples/            # Example source code files
│   ├── hello.lang       # Basic hello world example
│   └── loops.lang       # Loop examples
├── Cargo.toml          # Project configuration
└── README.md           # This file
```

## Features

### Compilation Pipeline
1. **Lexical Analysis** - Tokenizes source code into meaningful tokens
2. **Syntax Analysis** - Parses tokens into an Abstract Syntax Tree (AST)
3. **Semantic Analysis** - Type checking and symbol table management
4. **Code Generation** - Generates target code for multiple platforms

### Target Platforms
- **Bytecode** - Custom stack-based bytecode
- **JavaScript** - Transpiles to JavaScript
- **LLVM IR** - LLVM Intermediate Representation (planned)
- **Assembly** - Native assembly code (planned)

### Language Features
- Variables with type inference
- Functions with parameters and return types
- Control flow (if/else, while, for loops)
- Basic data types (Number, String, Boolean)
- Arithmetic and logical operations
- Function calls and recursion

## Usage

### Building the Project
```bash
cargo build --release
```

### Running the Compiler
```bash
# Compile to JavaScript (default)
cargo run examples/hello.lang

# Compile to specific target
cargo run examples/hello.lang javascript
cargo run examples/hello.lang bytecode
```

### Example Source Code

```rust
// variables_example.lang
fn main() {
    let x = 42;
    let message = "Hello, World!";
    let is_valid = true;
    
    if is_valid {
        print(message);
        print(x);
    }
}
```

## Development Roadmap

### Phase 1: Basic Infrastructure (Current)
- [x] Project structure setup
- [x] Basic AST definitions
- [x] Error handling framework
- [x] Lexer skeleton
- [x] Parser skeleton
- [x] Semantic analyzer skeleton
- [x] Code generator skeleton

### Phase 2: Core Functionality
- [ ] Complete lexer implementation
- [ ] Complete parser implementation
- [ ] Basic semantic analysis
- [ ] JavaScript code generation
- [ ] Standard library functions

### Phase 3: Advanced Features
- [ ] Advanced type system
- [ ] Optimizations
- [ ] LLVM backend
- [ ] Native compilation
- [ ] Standard library expansion

### Phase 4: Language Extensions
- [ ] Modules and imports
- [ ] Object-oriented features
- [ ] Generics/templates
- [ ] Memory management
- [ ] Concurrent programming

## Contributing

This is a learning project for understanding compiler design and implementation. Contributions and improvements are welcome!

## Architecture Notes

### Lexer (`lexer.rs`)
- Converts source text into tokens
- Handles keywords, operators, literals, identifiers
- Tracks position information for error reporting

### Parser (`parser.rs`)
- Implements recursive descent parsing
- Builds Abstract Syntax Tree from tokens
- Handles operator precedence and associativity

### AST (`ast.rs`)
- Defines all node types for expressions and statements
- Includes type annotations and metadata
- Designed for easy traversal and transformation

### Semantic Analyzer (`semantic.rs`)
- Symbol table management with scoping
- Type checking and inference
- Variable initialization tracking
- Function signature validation

### Code Generator (`codegen.rs`)
- Multi-target code generation
- Platform-specific optimizations
- Extensible architecture for new targets

### Error Handling (`error.rs`)
- Comprehensive error reporting
- Source location tracking
- Warning and error categorization
- User-friendly error messages

## License

MIT License - see LICENSE file for details. 