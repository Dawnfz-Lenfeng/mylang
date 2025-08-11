# Rust Interpreter

A simple interpreter built in Rust, reference from https://craftinginterpreters.com/.

## Features

- [x] Lexer
- [x] Parser
- [x] Treewalk Interpreter
- [x] VM with bytecode
- [ ] Object-oriented

## Usage

```bash
# Run the interpreter with the example script
cargo run examples/hello.myl --vm # with the VM
cargo run examples/hello.myl --tr # with the treewalk interpreter

# Run the interpreter with the interactive mode
cargo run # default with the treewalk interpreter
```

## Tests

There are four main test suites in this project, each targeting a core component:

```bash
cargo test lexer_tests
cargo test parser_tests
cargo test treewalk_tests
cargo test vm_tests
```

## Language Grammar

```
program -> statement*
```

### Statements

```
statement    -> exprStmt
                | forStmt
                | whileStmt
                | ifStmt
                | printStmt
                | returnStmt
                | breakStmt
                | continueStmt
                | varDecl
                | funcDecl
                | block

exprStmt     -> expression ';'
forStmt      -> 'for' ( varDecl | exprStmt | ';' ) expression? ';' expression? block
whileStmt    -> 'while' expression block
ifStmt       -> 'if' expression block ( 'else' ( ifStmt | block ) )?
printStmt    -> 'print' arguments? ';'
returnStmt   -> 'return' expression? ';'
breakStmt    -> 'break' ';'
continueStmt -> 'continue' ';'
varDecl      -> 'let' Identifier ( '=' expression )? ';'
funcDecl     -> 'fn' Identifier '(' parameters? ')' block
block        -> '{' statement* '}'
```

### Expressions

```
expression   -> assignment

assignment   -> ( Identifier | arrayAccess ) ( '=' | '+=' | '-=' | '*=' | '/=' ) assignment 
                | logic_or

logic_or     -> logic_and ( 'or' logic_and )*
logic_and    -> equality ( 'and' equality )*
equality     -> comparison ( ( '==' | '!=' ) comparison )*
comparison   -> term ( ( '<' | '>' | '<=' | '>=' ) term )*
term         -> factor ( ( '-' | '+' ) factor )*
factor       -> unary ( ( '/' | '*' ) unary )*
unary        -> ( '!' | '-' ) unary | call
call         -> primary ( '(' arguments? ')' | '[' expression ']' )*
arrayLiteral -> '[' arguments? ']'
arrayAccess  -> primary '[' expression ']'
primary      -> 'true' 
                | 'false'
                | 'nil' 
                | Number 
                | String 
                | Identifier 
                | arrayLiteral
                | '(' expression ')'
```

### Utils

```
arguments     -> expression ( ',' expression )*
parameters    -> Identifier ( ',' Identifier )*
```
