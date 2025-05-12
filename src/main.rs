use crate::command::CommandSpec;
use crate::error::Error;
use std::path::Path;
use std::{env, io, process};

mod command;
mod error;

const EXIT_OK: i32 = 0;
const EXIT_IO_ERROR: i32 = 1;
const EXIT_VERIFY_ERROR: i32 = 2;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() <= 1 {
        usage();
        process::exit(EXIT_OK);
    }
    let files = &args[1..];
    for f in files {
        let f = Path::new(f);
        let status_code = 0;
        let cmd_spec = CommandSpec::new(f, status_code);

        // We execute our test
        print_running(f);

        let ret = cmd_spec.execute();
        let result = match ret {
            Ok(result) => result,
            Err(err) => {
                print_io_error(err);
                print_failure(f);
                process::exit(EXIT_IO_ERROR);
            }
        };

        // Now we can verify against the expected value:
        let ret = cmd_spec.verify(&result);
        match ret {
            Ok(_) => print_success(f),
            Err(err) => {
                print_error(&err);
                print_failure(f);
                process::exit(EXIT_VERIFY_ERROR);
            }
        }
    }
    process::exit(EXIT_OK);
}

fn print_running(f: &Path) {
    eprintln!("{}: Running", f.display())
}

fn print_success(f: &Path) {
    eprintln!("{}: Success", f.display())
}

fn print_failure(f: &Path) {
    eprintln!("{}: Failure", f.display())
}

fn print_io_error(error: io::Error) {
    eprintln!("--> error: {error}");
}

fn print_error(error: &Error) {
    eprintln!("{}", error.render());
}

/// Prints command line usage.
fn usage() {
    println!("cliche, snapshot tests for CLIs.");
    println!();
    println!("cliche [FILES]...");
}
