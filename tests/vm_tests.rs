use mylang::run_with_vm;
use std::fs;

macro_rules! generate_example_tests {
    ($($test_name:ident => $file_name:literal),*) => {
        $(
            #[test]
            fn $test_name() {
                let filename = $file_name;
                let path = format!("examples/{}", filename);
                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Failed to read file: {}", path));
                let result = run_with_vm(source);
                assert!(
                    result.is_ok(),
                    "{filename} failed: {}",
                    result.err().unwrap()
                );
            }
        )*
    };
}

#[cfg(test)]
mod file_tests {
    use super::*;

    generate_example_tests!(
        test_arithmetic => "arithmetic.myl",
        test_arrays => "arrays.myl",
        test_break_continue => "break_continue.myl",
        test_builtins => "builtins.myl",
        test_complex_for_break_continue => "complex_for_break_continue.myl",
        test_complex_break_continue => "complex_break_continue.myl",
        test_complex_closures => "complex_closures.myl",
        test_compound_assignment => "compound_assignment.myl",
        test_conditionals => "conditionals.myl",
        test_edge_cases => "edge_cases.myl",
        test_else_if => "else_if.myl",
        test_enclosing => "enclosing.myl",
        test_factorial => "factorial.myl",
        test_fibonacci => "fibonacci.myl",
        test_functions => "functions.myl",
        test_hello => "hello.myl",
        test_loops => "loops.myl",
        test_scoping => "scoping.myl",
        test_short_circuit => "short_circuit.myl",
        test_variables => "variables.myl"
    );
}
