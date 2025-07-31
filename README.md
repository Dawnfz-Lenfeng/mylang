# Rust Interpreter

A simple interpreter built in Rust, reference from https://craftinginterpreters.com/.

## Usage

```bash
# Run the interpreter with the example script
cargo run examples/hello.myl
# Run the interpreter with the interactive mode
cargo run
```

## Tests

There are three main test suites in this project, each targeting a core component:

```bash
cargo test lexer_tests
cargo test parser_tests
cargo test interpreter_tests
```

## Language Grammar

```
program -> statement*
```

### Statements

```
statement -> exprStmt
            | forStmt
            | whileStmt
            | ifStmt
            | printStmt
            | returnStmt
            | varDecl
            | funcDecl
            | block

exprStmt   -> expression ';'
forStmt    -> 'for' ( varStmt | exprStmt | ';' )  expression? ';' ( expression )? block
whileStmt  -> 'while' '(' expression ')' block
ifStmt     -> 'if' expression block ( 'else' block )?
printStmt  -> 'print' expression ';'
returnStmt -> 'return' expression? ';'
varDecl    -> 'let' Identifier ( '=' expression )? ';'
funcDecl     -> 'fn' Identifier '(' parameters? ')' block
block      -> '{' statement* '}'
```

### Expressions

```
expression -> assignment

assignment -> Identifier '=' assignment | logic_or
logic_or   -> logic_and ( 'or' logic_and )*
logic_and  -> equality ( 'and' equality )*
equality   -> comparison ( ( '==' | '!=' ) comparison )*
comparison -> term ( ( '<' | '>' | '<=' | '>=' ) term )*
term       -> factor ( ( '-' | '+' ) factor )*
factor     -> unary ( ( '/' | '*' ) unary )*
unary      -> ( '!' | '-' ) unary | call
call       -> primary ( '(' arguments? ')' )*
primary    -> 'true' 
            | 'false'
            | 'nil' 
            | Number 
            | String 
            | Identifier 
            | '(' expression ')'
```

### Utils

```
arguments    -> expression ( ',' expression )*
parameters   -> Identifier ( ',' Identifier )*
```
