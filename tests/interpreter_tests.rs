use mylang::{
    error::Result, interpreter::Interpreter, lexer::lexer::Lexer, parser::parser::Parser,
};
use std::{cell::RefCell, fs, io::Write, rc::Rc};

struct OutputCapture {
    buffer: Rc<RefCell<Vec<u8>>>,
}

impl OutputCapture {
    fn new() -> (Self, Rc<RefCell<Vec<u8>>>) {
        let buffer = Rc::new(RefCell::new(Vec::new()));
        let capture = Self {
            buffer: buffer.clone(),
        };
        (capture, buffer)
    }
}

impl Write for OutputCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn run_myl_file(filename: &str) -> Result<String> {
    let source = fs::read_to_string(format!("examples/{}", filename))
        .expect(&format!("Failed to read file: examples/{}", filename));

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let (capture, buffer) = OutputCapture::new();
    let mut interpreter = Interpreter::with_output(Box::new(capture));

    interpreter.interpret(&program)?;

    let output_bytes = buffer.borrow().clone();
    Ok(String::from_utf8(output_bytes).unwrap())
}

#[cfg(test)]
mod file_tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let output = run_myl_file("hello.myl").unwrap();
        assert_eq!(output, "Hello, World!\n");
    }

    #[test]
    fn test_arithmetic_operations() {
        let output = run_myl_file("arithmetic.myl").unwrap();
        let expected = concat!(
            "Addition: 13\n",
            "Subtraction: 7\n",
            "Multiplication: 30\n",
            "Division: 3.3333333333333335\n",
            "Precedence test: 14\n",
            "Parentheses: 20\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_variables_and_scoping() {
        let output = run_myl_file("variables.myl").unwrap();
        let expected = concat!(
            "Number: 42\n",
            "Text: Hello\n",
            "Boolean: true\n",
            "Updated number: 100\n",
            "Updated text: World\n",
            "Updated boolean: false\n",
            "Inside block: I'm local\n",
            "Shadowed num: 999\n",
            "Outside block num: 100\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_conditionals() {
        let output = run_myl_file("conditionals.myl").unwrap();
        let expected = concat!(
            "x is greater than 5\n",
            "y is between 10 and 20\n",
            "Both conditions are true\n",
            "At least one condition is true\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_functions() {
        let output = run_myl_file("functions.myl").unwrap();
        let expected = concat!(
            "Hello, World\n",
            "Sum: 8\n",
            "Multiplying 4 and 6\n",
            "Product: 24\n",
            "This function returns nil\n",
            "Result: nil\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_factorial_calculation() {
        let output = run_myl_file("factorial.myl").unwrap();
        let expected = concat!("5! = \n", "120\n");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_fibonacci_sequence() {
        let output = run_myl_file("fibonacci.myl").unwrap();
        let expected = concat!(
            "fib 0 0\n",
            "fib 1 1\n",
            "fib 2 1\n",
            "fib 3 2\n",
            "fib 4 3\n",
            "fib 5 5\n",
            "fib 6 8\n",
            "fib 7 13\n",
            "fib 8 21\n",
            "fib 9 34\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_nested_loops() {
        let output = run_myl_file("loops.myl").unwrap();
        let expected = concat!(
            "Count: 0\n",
            "Count: 1\n",
            "Count: 2\n",
            "Count: 3\n",
            "Count: 4\n",
            "for loop\n",
            "Position: 0 0\n",
            "Position: 0 1\n",
            "Position: 0 2\n",
            "Position: 1 0\n",
            "Position: 1 1\n",
            "Position: 1 2\n",
            "Position: 2 0\n",
            "Position: 2 1\n",
            "Position: 2 2\n",
            "while loop\n",
            "Position: 0 0\n",
            "Position: 0 1\n",
            "Position: 0 2\n",
            "Position: 1 0\n",
            "Position: 1 1\n",
            "Position: 1 2\n",
            "Position: 2 0\n",
            "Position: 2 1\n",
            "Position: 2 2\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_function_scoping() {
        let output = run_myl_file("scoping.myl").unwrap();
        let expected = concat!(
            "In outer function: outer\n",
            "Can access global: global\n",
            "In inner function: inner\n",
            "Can access outer: outer\n",
            "Can access global: global\n",
            "Back in outer function\n",
            "Back in global scope: global\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_closures() {
        let output = run_myl_file("enclosing.myl").unwrap();
        let expected = concat!("0\n", "1\n");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_complex_closures() {
        let output = run_myl_file("complex_closures.myl").unwrap();
        let expected = concat!(
            "Counter1: 1\n",
            "Counter1: 2\n",
            "Counter2: 101\n",
            "Counter1: 3\n",
            "Counter2: 102\n",
            "Values: 1 2 3\n",
            "Nested result: 6\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_edge_cases() {
        let output = run_myl_file("edge_cases.myl").unwrap();
        let expected = concat!(
            "Empty function result: nil\n",
            "Zero: 0\n",
            "Negative: -42\n",
            "True and true: true\n",
            "True and false: false\n",
            "False or true: true\n",
            "Not true: false\n",
            "Not false: true\n",
            "Empty string: \n",
            "Space:  \n",
            "Nested values: 1 2 3\n",
            "After inner block: 1 2\n",
            "After middle block: 1\n",
            "Return 1: early return\n",
            "After if statement\n",
            "Return 2: second return\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_short_circuit_evaluation() {
        let output = run_myl_file("short_circuit.myl").unwrap();
        let expected = concat!(
            "=== Testing AND short circuit ===\n",
            "Result1: false Call count: 0\n",
            "Side effect called: should be called Count: 1\n",
            "Result2: true Call count: 1\n",
            "=== Testing OR short circuit ===\n",
            "Result3: true Call count: 0\n",
            "Side effect called: should be called Count: 1\n",
            "Result4: true Call count: 1\n",
            "=== Complex short circuit tests ===\n",
            "False side effect called: first Count: 1\n",
            "Result5: false Call count: 1\n",
            "Side effect called: first Count: 1\n",
            "Result6: true Call count: 1\n",
            "=== Variable assignment in short circuit ===\n",
            "Result7: false Assigned: false\n",
            "Result8: true Assigned: false\n"
        );
        assert_eq!(output, expected);
    }

    // Error handling tests
    #[test]
    fn test_undefined_variable_error() {
        let result = run_myl_file("error_undefined_var.myl");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("not defined"));
        }
    }

    #[test]
    fn test_type_mismatch_error() {
        let result = run_myl_file("error_type_mismatch.myl");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_not_found_error() {
        let result = run_myl_file("error_function_not_found.myl");
        assert!(result.is_err());
    }
}
