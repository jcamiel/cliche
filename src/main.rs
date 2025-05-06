use crate::command::{CommandSpec, VerifyError};
use std::io::Error;
use std::path::Path;
use std::{env, process};

mod command;

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
                print_verify_error(&err);
                print_failure(f);
                process::exit(EXIT_VERIFY_ERROR);
            }
        }
    }
    process::exit(EXIT_OK);
}

fn print_running(f: &Path) {
    eprintln!("{}: running", f.display())
}

fn print_success(f: &Path) {
    eprintln!("{}: success", f.display())
}

fn print_failure(f: &Path) {
    eprintln!("{}: failure", f.display())
}

fn print_io_error(error: Error) {
    eprintln!("--> error: {error}");
}

fn print_verify_error(error: &VerifyError) {
    match error {
        VerifyError::ExitCode { actual, expected } => {
            eprintln!("--> error: exit code not equals");
            eprintln!("    actual:   {}", actual);
            eprintln!("    expected: {}", expected);
        }
    }
}

/// Prints command line usage.
fn usage() {
    println!("cliche, snapshot tests for CLIs.");
    println!();
    println!("cliche [FILES]...");
}
