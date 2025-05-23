use crate::command::ExitCode;
use crate::text::{Format, Style, StyledString};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// There is an issue reading file.
    FileRead { path: PathBuf, cause: String },
    /// The file is not a valid UTF-8 string.
    FileNotUtf8 { path: PathBuf },
    /// The file can't be read as an integer (used for expected exit code).
    FileNotInteger { path: PathBuf },
    /// The expected exit code and the actual exit code are not equals.
    CheckExitCode {
        cmd_path: PathBuf,
        expected: ExitCode,
        actual: ExitCode,
        stderr: Vec<u8>,
    },
    /// A line in actual stdout doesn't equal the expected stdout line.
    CheckStdoutLine {
        cmd_path: PathBuf,
        expected: Option<String>,
        actual: Option<String>,
        /// 1-based line index.
        row: usize,
    },
    /// A line in actual stdout doesn't match the expected stdout pattern.
    CheckStdoutPattern {
        cmd_path: PathBuf,
        expected: Option<String>,
        actual: Option<String>,
        /// 1-based line index.
        row: usize,
    },
    /// A pattern stdout file is not valid
    StdoutPatternFileInvalid {
        cmd_path: PathBuf,
        reason: String,
        /// 1-based line index.
        row: usize,
    },
    /// A line in actual stderr doesn't equal the expected stderr line.
    CheckStderrLine {
        cmd_path: PathBuf,
        expected: Option<String>,
        actual: Option<String>,
        /// 1-based line index.
        row: usize,
    },
}

impl Error {
    pub fn render(&self) -> String {
        match self {
            Error::FileRead { .. } => "--> error FileRead".to_string(),
            Error::FileNotUtf8 { .. } => "--> error FileNotUtf8".to_string(),
            Error::FileNotInteger { .. } => "--> error FileNotInteger".to_string(),
            Error::CheckExitCode { cmd_path, expected, actual, .. } => {
                let title = "Exit code doesn't match";
                let script_title   = "  script  :";
                let expected_title = "  expected:";
                let actual_title   = "  actual  :";
                diff_exit(
                    &title,
                    script_title,
                    cmd_path,
                    expected_title,
                    *expected,
                    actual_title,
                    *actual,
                    Format::Ansi,
                )
            },
            Error::CheckStdoutLine {
                cmd_path,
                expected,
                actual,
                row,
            } => {
                let title = format!("Stdout doesn't match at line {}", row);
                let script_title = "  script       :";
                let expected_title = "  expected line:";
                let actual_title = "  actual line  :";
                diff_text(
                    &title,
                    script_title,
                    cmd_path,
                    expected_title,
                    expected.as_deref(),
                    actual_title,
                    actual.as_deref(),
                    Format::Ansi,
                )
            }
            Error::CheckStdoutPattern {
                cmd_path,
                expected,
                actual,
                row,
            } => {
                let title = format!("Stdout doesn't match at line {}", row);
                let script_title = "  script          :";
                let expected_title = "  expected pattern:";
                let actual_title = "  actual line     :";
                diff_text(
                    &title,
                    script_title,
                    cmd_path,
                    expected_title,
                    expected.as_deref(),
                    actual_title,
                    actual.as_deref(),
                    Format::Ansi,
                )
            }
            Error::CheckStderrLine {
                cmd_path,
                expected,
                actual,
                row,
            } => {
                let title = format!("Stderr doesn't match at line {}", row);
                let script_title = "  script       :";
                let expected_title = "  expected line:";
                let actual_title = "  actual line  :";
                diff_text(
                    &title,
                    script_title,
                    cmd_path,
                    expected_title,
                    expected.as_deref(),
                    actual_title,
                    actual.as_deref(),
                    Format::Ansi,
                )
            }
            Error::StdoutPatternFileInvalid { .. } => {
                "--> error StdoutPatternFileInvalid".to_string()
            }
        }
    }
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

#[allow(clippy::too_many_arguments)]
fn diff_text(
    title: &str,
    script_title: &str,
    script: &Path,
    expected_title: &str,
    expected: Option<&str>,
    actual_title: &str,
    actual: Option<&str>,
    format: Format,
) -> String {
    let red_bold = Style::new().red().bold();
    let bold = Style::new().bold();
    let blue_bold = Style::new().blue().bold();
    let yellow = Style::new().yellow();

    let mut s = StyledString::new();
    s.push_with("error", red_bold);
    s.push_with(":", bold);
    s.push(" ");
    s.push_with(title, bold);
    s.push("\n");
    s.push_with(script_title, blue_bold);
    s.push(" ");
    s.push(&script.display().to_string());
    s.push("\n");

    let expected = expected.unwrap_or("");
    let expected = replace_visible(expected);
    s.push_with(expected_title, blue_bold);
    s.push(" ");
    s.push_with("<", yellow);
    s.push(&expected);
    s.push_with(">", yellow);
    s.push("\n");

    let actual = actual.unwrap_or("");
    let actual = replace_visible(actual);
    s.push_with(actual_title, blue_bold);
    s.push(" ");
    s.push_with("<", yellow);
    s.push(&actual);
    s.push_with(">", yellow);
    s.push("\n");
    s.to_string(format)
}


fn diff_exit(
    title: &str,
    script_title: &str,
    script: &Path,
    expected_title: &str,
    expected: ExitCode,
    actual_title: &str,
    actual: ExitCode,
    format: Format,
) -> String {
    let red_bold = Style::new().red().bold();
    let bold = Style::new().bold();
    let blue_bold = Style::new().blue().bold();

    let mut s = StyledString::new();
    s.push_with("error", red_bold);
    s.push_with(":", bold);
    s.push(" ");
    s.push_with(title, bold);
    s.push("\n");
    s.push_with(script_title, blue_bold);
    s.push(" ");
    s.push(&script.display().to_string());
    s.push("\n");

    s.push_with(expected_title, blue_bold);
    s.push(" ");
    s.push(&expected.to_string());
    s.push("\n");

    s.push_with(actual_title, blue_bold);
    s.push(" ");
    s.push(&actual.to_string());
    s.push("\n");
    s.to_string(format)
}
