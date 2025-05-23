use crate::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fmt, fs, io};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExitCode(i32);

impl ExitCode {
    #[allow(dead_code)]
    pub fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for ExitCode {
    fn from(value: i32) -> Self {
        ExitCode(value)
    }
}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}


/// Represents a command specification
pub struct CommandSpec {
    cmd_path: PathBuf,
    stdout_path: Option<PathBuf>,
    stdout_pat_path: Option<PathBuf>,
    stderr_path: Option<PathBuf>,
    exit_code_path: Option<PathBuf>,
}

impl CommandSpec {
    /// Creates a new expected command spec using script at `cmd_path`.
    pub fn new(cmd_path: &Path) -> Result<Self, io::Error> {
        let cmd_path = fs::canonicalize(cmd_path)?;
        let stdout_path = with_ext(&cmd_path, "out");
        let stdout_pat_path = with_ext(&cmd_path, "out.pattern");
        let exit_code_path = with_ext(&cmd_path, "exit");
        let stderr_path = with_ext(&cmd_path, "err");

        Ok(CommandSpec {
            cmd_path,
            stdout_path,
            stdout_pat_path,
            stderr_path,
            exit_code_path,
        })
    }

    /// Executes the command and returns the result.
    pub fn execute(&self) -> Result<CommandResult, io::Error> {
        let output = Command::new(self.cmd_path.as_os_str()).output()?;
        let exit_code = output.status.code().unwrap();
        let exit_code = ExitCode(exit_code);
        let stdout = &output.stdout;
        let stderr = &output.stderr;
        Ok(CommandResult::new(exit_code, stdout, stderr))
    }

    /// Returns the expected code for this command spec.
    pub fn exit_code(&self) -> Result<ExitCode, Error> {
        let Some(exit_code_path) = &self.exit_code_path else {
            return Ok(ExitCode(0));
        };

        let exit_code = match fs::read(exit_code_path) {
            Ok(s) => s,
            Err(err) => {
                return Err(Error::FileRead {
                    path: exit_code_path.clone(),
                    cause: err.to_string(),
                });
            }
        };
        let Ok(exit_code) = String::from_utf8(exit_code.clone()) else {
            return Err(Error::FileNotUtf8 {
                path: exit_code_path.clone(),
            });
        };
        let exit_code = exit_code.trim();
        let Ok(exit_code) = exit_code.parse::<i32>() else {
            return Err(Error::FileNotInteger {
                path: exit_code_path.clone(),
            });
        };
        Ok(ExitCode(exit_code))
    }

    /// Returns `true` if this command has expected stdout, `false` otherwise.
    pub fn has_stdout(&self) -> bool {
        self.stdout_path.is_some()
    }

    /// Returns the expected stdout buffer for this command spec.
    pub fn stdout(&self) -> Result<Vec<u8>, Error> {
        let Some(stdout_path) = &self.stdout_path else {
            return Ok(vec![]);
        };
        let stdout = match fs::read(stdout_path) {
            Ok(s) => s,
            Err(err) => {
                return Err(Error::FileRead {
                    path: stdout_path.clone(),
                    cause: err.to_string(),
                });
            }
        };
        Ok(stdout)
    }

    /// Returns `true` if this command has expected stdout, `false` otherwise.
    pub fn has_stdout_pat(&self) -> bool {
        self.stdout_pat_path.is_some()
    }

    /// Returns the expected patterned stdout buffer for this command spec.
    /// For the moment, we only deal with UTF-8 pattern stdout
    pub fn stdout_pat(&self) -> Result<String, Error> {
        let Some(stdout_pat_path) = &self.stdout_pat_path else {
            return Ok("".to_string());
        };
        let stdout_pat = match fs::read(stdout_pat_path) {
            Ok(s) => s,
            Err(err) => {
                return Err(Error::FileRead {
                    path: stdout_pat_path.clone(),
                    cause: err.to_string(),
                });
            }
        };
        let Ok(stdout_pat) = String::from_utf8(stdout_pat) else {
            return Err(Error::FileNotUtf8 {
                path: stdout_pat_path.clone(),
            });
        };
        Ok(stdout_pat)
    }

    pub fn has_stderr(&self) -> bool {
        self.stderr_path.is_some()
    }

    /// Returns the expected stderr buffer for this command spec.
    pub fn stderr(&self) -> Result<Vec<u8>, Error> {
        let Some(stderr_path) = &self.stderr_path else {
            return Ok(vec![]);
        };
        let stderr = match fs::read(stderr_path) {
            Ok(s) => s,
            Err(err) => {
                return Err(Error::FileRead {
                    path: stderr_path.clone(),
                    cause: err.to_string(),
                });
            }
        };
        Ok(stderr)
    }

    pub fn cmd_path(&self) -> &Path {
        &self.cmd_path
    }
}

#[allow(dead_code)]
pub struct CommandResult {
    exit_code: ExitCode,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl CommandResult {
    pub fn new(exit_code: ExitCode, stdout: &[u8], stderr: &[u8]) -> Self {
        CommandResult {
            exit_code,
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }

    pub fn exit_code(&self) -> ExitCode {
        self.exit_code
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

fn with_ext(path: &Path, ext: &str) -> Option<PathBuf> {
    let mut path = path.to_path_buf();
    path.set_extension(ext);
    if path.exists() { Some(path) } else { None }
}
