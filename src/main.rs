use mylang::{print_usage, run_file, run_prompt};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        run_prompt();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        print_usage(&args[0]);
        std::process::exit(1);
    }
}
