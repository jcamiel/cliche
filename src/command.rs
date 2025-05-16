use crate::error::Error;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};
use crate::split;

pub struct CommandSpec {
    cmd_path: PathBuf,
    stdout_path: PathBuf,
    stdout_pat_path: PathBuf,
    /// Expected exit code of the command.
    exit_code: i32,
}

impl CommandSpec {
    pub fn new(cmd: &Path, exit_code: i32) -> Self {
        let mut stdout_path = PathBuf::from(cmd);
        stdout_path.set_extension("out");

        let mut stdout_pat_path = PathBuf::from(cmd);
        stdout_pat_path.set_extension("out.pattern");

        CommandSpec {
            cmd_path: cmd.to_path_buf(),
            stdout_path,
            stdout_pat_path,
            exit_code,
        }
    }

    pub fn execute(&self) -> Result<CommandResult, io::Error> {
        let cmd = fs::canonicalize(&self.cmd_path)?;
        let output = Command::new(cmd.as_os_str()).output()?;
        let exit_code = output.status.code().unwrap();
        let stdout = &output.stdout;
        let stderr = &output.stderr;
        Ok(CommandResult::new(exit_code, stdout, stderr))
    }

    /// Checks the result of an executed command against this commabd spec.
    pub fn verify(&self, result: &CommandResult) -> Result<(), Error> {
        // Check for the status code:
        if self.exit_code != result.exit_code {
            let err = Error::ExitCodeCheck {
                expected: self.exit_code,
                actual: result.exit_code,
                stderr: result.stderr.clone(),
            };
            return Err(err);
        }

        // Check for stdout:
        if let Ok(stdout_path) = fs::canonicalize(&self.stdout_path) {
            if stdout_path.exists() {
                let expected_stdout = match fs::read(stdout_path) {
                    Ok(s) => s,
                    Err(err) => {
                        return Err(Error::FileRead {
                            path: self.stdout_path.clone(),
                            cause: err.to_string(),
                        });
                    }
                };

                if expected_stdout != result.stdout {
                    let err = Error::StdoutCheck {
                        expected: expected_stdout,
                        actual: result.stdout.clone(),
                    };
                    return Err(err);
                }
            }
        }

        // Check for stdout pattern
        if let Ok(pat_path) = fs::canonicalize(&self.stdout_pat_path) {
            if pat_path.exists() {
                let expected_pat = match fs::read(pat_path) {
                    Ok(s) => s,
                    Err(err) => {
                        return Err(Error::FileRead {
                            path: self.stdout_pat_path.clone(),
                            cause: err.to_string(),
                        });
                    }
                };

                // Transform or pattern to a regex. Pattern must be UTF8 strings
                // TODO: manage errors
                let expected_pat = String::from_utf8(expected_pat.clone()).unwrap();
                let actual = String::from_utf8(result.stdout.clone()).unwrap();

                let expected_pat_lines = expected_pat.split('\n').collect::<Vec<_>>();
                let actual_lines = actual.split('\n').collect::<Vec<_>>();
                if expected_pat_lines.len() != actual_lines.len() {
                    return Err(Error::StdoutPatternLinesCount);
                }

                let mut row = 0;

                for line_pat in expected_pat_lines.iter() {

                    // TODO if the line does not contains pattern, do a simple match
                    // TODO: Test if pattern is valid
                    let line_pat = parse_pattern(line_pat);
                    let re = Regex::new(&line_pat).unwrap();
                    let line = actual_lines[row];
                    if !re.is_match(line) {
                        println!("row: {}", row);
                        println!("line pattern: <{}>", line_pat);
                        println!("line: <{}>", line);
                        let err = Error::StdoutPatternCheck;
                        return Err(err);
                    }
                    row += 1;
                }
            }
        };

        Ok(())
    }
}

#[allow(dead_code)]
pub struct CommandResult {
    exit_code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl CommandResult {
    fn new(exit_code: i32, stdout: &[u8], stderr: &[u8]) -> Self {
        CommandResult {
            exit_code,
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }
}

/// Create a standard regex string from the custom pattern file.
fn parse_pattern(s: &str) -> String {
    // Split string into alternating tokens: escaping regex metacharacters and no escaping
    // The first token must always be escaped
    let tokens = split::split_capture(r"<<<([^>]+)>>>", s);
    let mut s = String::from('^');
    for (i, token) in tokens.iter().enumerate() {
        if i % 2 == 0 {
            let escape = escape_regex_metacharacters(token);
            s.push_str(&escape);
        } else {
            s.push_str(token);
        }
    }
    s.push('$');
    s
}

/// Escape all regex metacharacters in a string
fn escape_regex_metacharacters(s: &str) -> String {
    let mut escaped_s = String::new();
    for c in s.chars() {
        let escaped_c = escape_regex_metacharacter(c);
        escaped_s.push_str(&escaped_c);
    }
    escaped_s
}

fn escape_regex_metacharacter(c: char) -> String {
    let meta = [
        '\\', '/', '.', '{', '}', '(', ')', '[', ']', '^', '$', '*', '+', '?', '|',
    ];
    if meta.contains(&c) {
        format!("\\{c}")
    } else {
        c.to_string()
    }
}
