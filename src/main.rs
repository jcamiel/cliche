use crate::command::CommandSpec;
use crate::error::Error;
use crate::text::{Format, Style, StyledString, init_crate_colored};
use std::path::Path;
use std::{env, io, process};

mod command;
mod error;
mod split;
mod text;

const EXIT_OK: i32 = 0;
const EXIT_IO_ERROR: i32 = 1;
const EXIT_VERIFY_ERROR: i32 = 2;

fn main() {
    init_crate_colored();

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
    let mut s = StyledString::new();
    s.push_with("Running", Style::new().cyan().bold());
    s.push(" ");
    s.push_with(&f.display().to_string(), Style::new().bold());
    eprintln!("{}", s.to_string(Format::Ansi));
}

fn print_success(f: &Path) {
    let mut s = StyledString::new();
    s.push_with("Success", Style::new().green().bold());
    s.push(" ");
    s.push_with(&f.display().to_string(), Style::new().bold());
    eprintln!("{}", s.to_string(Format::Ansi));
}

fn print_failure(f: &Path) {
    let mut s = StyledString::new();
    s.push_with("Failure", Style::new().red().bold());
    s.push(" ");
    s.push_with(&f.display().to_string(), Style::new().bold());
    eprintln!("{}", s.to_string(Format::Ansi));
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
