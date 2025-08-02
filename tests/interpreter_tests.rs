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

    #[test]
    fn test_compound_assignment_operators() {
        let output = run_myl_file("compound_assignment.myl").unwrap();
        let expected = concat!(
            "Initial x: 10\n",
            "After x += 5: 15\n",
            "After x -= 3: 12\n",
            "After x *= 2: 24\n",
            "After x /= 4: 6\n",
            "y += x * 2 (y was 100, x is 6): 112\n",
            "After a += b += 2: a = 10 b = 5\n",
            "Count after three increments: 3\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_array_functionality() {
        let output = run_myl_file("arrays.myl").unwrap();
        let expected = concat!(
            "Numbers array: [1, 2, 3, 4, 5]\n",
            "Mixed array: [42, hello, true, [1, 2]]\n",
            "Empty array: []\n",
            "First number: 1\n",
            "Last number: 5\n",
            "Mixed second element: hello\n",
            "After changing first element: [100, 2, 3, 4, 5]\n",
            "After replacing nested array: [42, hello, true, replaced nested array]\n",
            "Calculated array: [30, 200, -10]\n",
            "After numbers[1] += 5: [100, 7, 3, 4, 5]\n",
            "After numbers[2] *= 3: [100, 7, 9, 4, 5]\n",
            "Matrix: [[1, 2], [3, 4], [5, 6]]\n",
            "Matrix[1][0]: 3\n",
            "Sum of test_array: 60\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_builtin_functions() {
        let output = run_myl_file("builtins.myl").expect("Failed to run builtins.myl");
        let expected = concat!(
            "Testing len() function:\n",
            "len([1, 2, 3, 4, 5]) = 5\n",
            "len('hello') = 5\n",
            "len([]) = 0\n",
            "len('') = 0\n",
            "Testing type() function:\n",
            "type(42) = number\n",
            "type('hello') = string\n",
            "type(true) = boolean\n",
            "type([1, 2, 3]) = array\n",
            "type(nil) = nil\n",
            "type(test_func) = function\n",
            "type(len) = builtin_function\n"
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

    #[test]
    fn test_array_bounds_error() {
        let result = run_myl_file("error_array_bounds.myl");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("out of bounds") || error.message.contains("index"));
        }
    }

    #[test]
    fn test_index_non_array_error() {
        let result = run_myl_file("error_index_non_array.myl");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("Cannot index") || error.message.contains("non-array"));
        }
    }

    #[test]
    fn test_break_continue_basic() {
        let output = run_myl_file("break_continue.myl").unwrap();
        let expected = concat!(
            "Testing break:\n",
            "i = 0\n",
            "i = 1\n",
            "i = 2\n",
            "After while loop, i = 3\n",
            "\n",
            "Testing continue:\n",
            "j = 1\n",
            "j = 2\n",
            "j = 4\n",
            "j = 5\n",
            "\n",
            "Testing nested break:\n",
            "x = 0 y = 0\n",
            "x = 1 y = 0\n",
            "x = 2 y = 0\n",
            "\n",
            "Testing nested continue:\n",
            "a = 0 b = 1\n",
            "a = 0 b = 3\n",
            "a = 1 b = 1\n",
            "a = 1 b = 3\n",
            "a = 2 b = 1\n",
            "a = 2 b = 3\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_complex_break_continue() {
        let output = run_myl_file("complex_break_continue.myl").unwrap();
        let expected = concat!(
            "=== Complex break/continue scenarios ===\n",
            "Test 1: Multiple conditions\n",
            "Processing 1\n",
            "Skipping 2\n",
            "Processing 3\n",
            "Processing 4\n",
            "Skipping 5 too\n",
            "Processing 6\n",
            "Processing 7\n",
            "Breaking at 8\n",
            "\n",
            "Test 2: With function calls\n",
            "Number: 1\n",
            "Number: 2\n",
            "Number: 4\n",
            "Number: 5\n",
            "Number: 7\n",
            "\n",
            "Test 3: Deeply nested\n",
            "Outer loop: 1\n",
            "Values: 1 1 1\n",
            "Values: 1 1 2\n",
            "Values: 1 1 3\n",
            "Values: 1 3 1\n",
            "Breaking inner at 1 3 2\n",
            "Breaking middle at 1 3\n",
            "Outer loop: 2\n",
            "Values: 2 1 1\n",
            "Values: 2 1 2\n",
            "Values: 2 1 3\n",
            "Values: 2 3 1\n",
            "Breaking inner at 2 3 2\n",
            "Breaking middle at 2 3\n",
            "Outer loop: 3\n",
            "Values: 3 1 1\n",
            "Values: 3 1 2\n",
            "Values: 3 1 3\n",
            "Values: 3 3 1\n",
            "Breaking inner at 3 3 2\n",
            "Breaking middle at 3 3\n"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_break_outside_loop_error() {
        let result = run_myl_file("error_break_outside_loop.myl");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("break") && error.message.contains("outside"));
        }
    }

    #[test]
    fn test_continue_outside_loop_error() {
        let result = run_myl_file("error_continue_outside_loop.myl");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("continue") && error.message.contains("outside"));
        }
    }

    #[test]
    fn test_break_in_function_error() {
        let result = run_myl_file("error_break_in_function.myl");
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.message.contains("break") && error.message.contains("outside"));
        }
    }
}
