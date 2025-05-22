use crate::command::CommandSpec;
use crate::error::Error;
use crate::text::{Format, Style, StyledString, init_crate_colored};
use std::path::Path;
use std::{env, io, process};

mod chunk;
mod command;
mod error;
mod text;
mod verify;

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

        print_running(f);

        let cmd_spec = CommandSpec::new(f);
        let cmd_spec = match cmd_spec {
            Ok(c) => c,
            Err(err) => {
                clear();
                print_io_error(err);
                print_failure(f);
                process::exit(EXIT_IO_ERROR);
            }
        };

        // We execute our test
        let cmd_result = cmd_spec.execute();
        let cmd_result = match cmd_result {
            Ok(c) => c,
            Err(err) => {
                clear();
                print_io_error(err);
                print_failure(f);
                process::exit(EXIT_IO_ERROR);
            }
        };

        // Now we can verify against the expected value:
        let check = verify::check_result(&cmd_spec, &cmd_result);
        match check {
            Ok(_) => {
                clear();
                print_success(f);
            }
            Err(err) => {
                clear();
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

fn clear() {
    eprint!("\x1B[1A\x1B[K");
}
/// Prints command line usage.
fn usage() {
    println!("cliche, snapshot tests for CLIs.");
    println!();
    println!("cliche [FILES]...");
}
