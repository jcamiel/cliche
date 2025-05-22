use crate::command::ExitCode;
use crate::text::{Format, Style, StyledString};
use std::cmp::max;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// There is an issue reading expected exit code (`foo.exit`).
    ExpectedExitCodeFile {
        path: PathBuf,
        cause: String,
    },
    FileRead {
        path: PathBuf,
        cause: String,
    },
    /// The expected exit code and the actual exit code are not equals.
    ExitCodeCheck {
        expected: ExitCode,
        actual: ExitCode,
        stderr: Vec<u8>,
    },
    /// A chunk of line is different between the expected sdtout and the actual sdtout.
    StdoutLineCheck {
        cmd_path: PathBuf,
        expected: Option<String>,
        actual: Option<String>,
        /// 1-based line index.
        line: u64,
    },
    StdoutPatternLinesCount,
    StdoutPatternCheck,
}

impl Error {
    pub fn render(&self) -> String {
        match self {
            Error::ExpectedExitCodeFile { .. } => "--> error ExpectedExitCodeFile".to_string(),
            Error::FileRead { .. } => "--> error FileRead".to_string(),
            Error::ExitCodeCheck { .. } => "--> error ExitCodeCheck".to_string(),
            Error::StdoutLineCheck {
                cmd_path,
                expected,
                actual,
                line,
            } => {
                let title = format!("Error stdout difference line {}", line);
                let red_bold = Style::new().red().bold();
                let bold = Style::new().bold();
                let blue_bold = Style::new().blue().bold();
                let yellow = Style::new().yellow();

                let mut s = StyledString::new();
                s.push_with("error", red_bold);
                s.push_with(":", bold);
                s.push(" ");
                s.push_with(&title, bold);
                s.push("\n");
                s.push_with("  script  :", blue_bold);
                s.push(" ");
                s.push(&cmd_path.display().to_string());
                s.push("\n");

                let expected = expected.clone().unwrap_or("".to_string());
                let expected = replace_visible(&expected);
                s.push_with("  expected:", blue_bold);
                s.push(" ");
                s.push_with("<", yellow);
                s.push(&expected);
                s.push_with(">", yellow);
                s.push("\n");

                let actual = actual.clone().unwrap_or("".to_string());
                let actual = replace_visible(&actual);
                s.push_with("  actual  :", blue_bold);
                s.push(" ");
                s.push_with("<", yellow);
                s.push(&actual);
                s.push_with(">", yellow);
                s.push("\n");
                s.to_string(Format::Ansi)
            }
            Error::StdoutPatternLinesCount => "--> error: StdoutPatternLinesCount".to_string(),
            Error::StdoutPatternCheck => "--> error: StdoutPatternCheck".to_string(),
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
            // add colors on first diff
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

    str.replace('\n', &lf)
        .replace('\r', &cr)
        .replace('\t', &tab)
}
