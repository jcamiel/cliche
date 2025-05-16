use std::cmp::max;
use std::path::PathBuf;
use crate::text::{Format, Style, StyledString};

pub enum Error {
    FileRead { path: PathBuf, cause: String },
    ExitCodeCheck { expected: i32, actual: i32, stderr: Vec<u8>},
    StdoutCheck { expected: Vec<u8>, actual: Vec<u8> },
}

impl Error {
    pub fn render(&self) -> String {
        match self {
            Error::FileRead { path, cause } => {
                let path = path.display();
                format!("--> error: {path} {cause}")
            }
            Error::ExitCodeCheck { actual, expected, stderr } => {
                // TODO: write sdterr
                let blue = Style::new().blue().bold();
                let mut error = StyledString::new();

                error.push_with("--> error", Style::new().bold().red());
                error.push_with(": exit code not equals", Style::new().bold());
                error.push("\n");
               error.push_with("actual:", blue);
                error.push("   ");
                error.push(&actual.to_string());
                error.push("\n");
                error.push_with("expected:", blue);
                error.push(" ");
                error.push(&expected.to_string());
                error.push("\n");
                error.push_with("stderr:", blue);
                let error = error.to_string(Format::Ansi);

                // TODO: manage error on stderr to text
                let stderr = String::from_utf8(stderr.clone()).unwrap();
                let mut separator = StyledString::new();
                separator.push_with("|", blue);
                let separator = separator.to_string(Format::Ansi);
                let stderr = stderr
                    .lines()                              // Split by newline
                    .map(|line| format!("{} {}", separator, line))     // Add '|' to each line
                    .collect::<Vec<_>>()                  // Collect into a Vec<String>
                    .join("\n");
                format!("{error}\n{stderr}\n")
            }
            Error::StdoutCheck { actual, expected } => {
                // We try to convert expected to string
                match String::from_utf8(expected.clone()) {
                    Ok(expected_str) => match String::from_utf8(actual.clone()) {
                        Ok(actual_str) => render_stdout_diff_str(&actual_str, &expected_str),
                        Err(_) => render_stdout_diff_bytes(actual, expected),
                    },
                    Err(_) => render_stdout_diff_bytes(actual, expected),
                }
            }
        }
    }
}

/// Renders a difference error between two string stdout.
fn render_stdout_diff_str(actual: &str, expected: &str) -> String {
    // Find first line differences. We split on \n so \r\n differences will be seen
    let actual = actual.split_inclusive('\n').collect::<Vec<_>>();
    let expected = expected.split_inclusive('\n').collect::<Vec<_>>();
    let max_lines = max(actual.len(), expected.len());
    for i in 0..max_lines {
        let actual_line = actual.get(i);
        let expected_line = expected.get(i);
        if actual_line != expected_line {
            let actual = match actual_line {
                Some(s) => s,
                None => "-",
            };
            let expected = match expected_line {
                Some(s) => s,
                None => "-",
            };
            // Replace invisible chars with some placeholder
            // TODO: manage all invisible
            // add coulors on first diff
            let actual = replace_visible(actual);
            let expected = replace_visible(expected);

            return format!(
                "--> error: stdout not equals (first difference on line {})\n\
                     actual:   <{actual}>\n\
                     expected: <{expected}>\n\
                ",
                i + 1
            );
        }
    }
    panic!("difference not found")
}

/// Renders a difference error between two bytes stdout.
fn render_stdout_diff_bytes(actual: &[u8], expected: &[u8]) -> String {
    let max_bytes = max(actual.len(), expected.len());
    for i in 0..max_bytes {
        let actual_byte = actual.get(i);
        let expected_byte = expected.get(i);
        if actual_byte != expected_byte {
            let actual = match actual_byte {
                Some(s) => format!("{:02x}", s),
                None => "-".to_string(),
            };
            let expected = match expected_byte {
                Some(s) => format!("{:02x}", s),
                None => "-".to_string(),
            };
            return format!(
                "--> error: stdout not equals (first difference on byte {}\n\
                     actual:   {actual}\n\
                     expected: {expected}\n\
                ",
                i + 1
            );
        }
    }
    panic!("difference not found")
}

fn replace_visible(str: &str) -> String {
    let yellow = Style::new().yellow();

    let mut lf = StyledString::new();
    lf.push_with("[\\n]", yellow);
    let lf = lf.to_string(Format::Ansi);

    let mut cr = StyledString::new();
    cr.push_with("[\\r]", yellow);
    let cr = cr.to_string(Format::Ansi);

    let mut tab = StyledString::new();
    tab.push_with("[\\tab]", yellow);
    let tab = tab.to_string(Format::Ansi);

    str.replace('\n', &lf).replace('\r', &cr).replace('\t', &tab)
}