use mylang::{print_usage, run_file_with_tr, run_file_with_vm, run_prompt};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_] => run_prompt(),
        [_, option] if option == "--help" => print_usage(&args[0]),
        [_, filename] => run_file_with_vm(filename),
        [_, filename, option] if option == "--tr" => run_file_with_tr(filename),
        [_, filename, option] if option == "--vm" => run_file_with_vm(filename),
        _ => {
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}
