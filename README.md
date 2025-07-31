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

### Declarations

```
declaration -> varDecl 
            | functionDecl
            | statement

varDecl     -> 'let' Identifier ( '=' expression )? ';'
fnDecl      -> 'fn' Identifier '(' parameters? ')' block
```

### Statements

```
statement -> exprStmt
            | forStmt
            | ifStmt
            | printStmt
            | returnStmt
            | varStmt
            | whileStmt
            | block

exprStmt   -> expression ';'
forStmt    -> 'for' '(' ( varStmt | exprStmt | ';' )  expression? ';' ( expression )? ')' statement
ifStmt     -> 'if' '(' expression ')' statement ( 'else' statement )?
printStmt  -> 'print' expression ';'
returnStmt -> 'return' expression? ';'
varStmt    -> 'let' Identifier ( '=' expression )? ';'
whileStmt  -> 'while' '(' expression ')' statement
block      -> '{' declaration* '}'
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
unary      -> ( ( '!' | '-' ) )* call
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
