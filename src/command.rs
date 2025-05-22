use crate::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExitCode(i32);

impl ExitCode {
    pub fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for ExitCode {
    fn from(value: i32) -> Self {
        ExitCode(value)
    }
}
/// Represents a command specification
pub struct CommandSpec {
    cmd_path: PathBuf,
    stdout_path: Option<PathBuf>,
    stdout_pat_path: Option<PathBuf>,
    exit_code_path: Option<PathBuf>,
}

impl CommandSpec {
    /// Creates a new expected command spec using script at `cmd_path`.
    pub fn new(cmd_path: &Path) -> Result<Self, io::Error> {
        let cmd_path = fs::canonicalize(cmd_path)?;

        let mut stdout_path = cmd_path.clone();
        stdout_path.set_extension("out");
        let stdout_path = if stdout_path.exists() {
            Some(stdout_path)
        } else {
            None
        };

        let mut stdout_pat_path = cmd_path.clone();
        stdout_pat_path.set_extension("out.pattern");
        let stdout_pat_path = if stdout_pat_path.exists() {
            Some(stdout_pat_path)
        } else {
            None
        };

        let mut exit_code_path = cmd_path.clone();
        exit_code_path.set_extension("exit");
        let exit_code_path = if exit_code_path.exists() {
            Some(exit_code_path)
        } else {
            None
        };

        Ok(CommandSpec {
            cmd_path,
            stdout_path,
            stdout_pat_path,
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
    pub fn expected_exit_code(&self) -> Result<ExitCode, Error> {
        let Some(exit_code_path) = &self.exit_code_path else {
            return Ok(ExitCode(0));
        };

        let exit_code = match fs::read(exit_code_path) {
            Ok(s) => s,
            Err(err) => {
                return Err(Error::ExpectedExitCodeFile {
                    path: exit_code_path.clone(),
                    cause: err.to_string(),
                });
            }
        };
        let Ok(exit_code) = String::from_utf8(exit_code.clone()) else {
            return Err(Error::ExpectedExitCodeFile {
                path: exit_code_path.clone(),
                cause: "`.exit` file encoding must use UTF-8".to_string(),
            });
        };
        let exit_code = exit_code.trim();
        let Ok(exit_code) = exit_code.parse::<i32>() else {
            return Err(Error::ExpectedExitCodeFile {
                path: exit_code_path.clone(),
                cause: "`.exit` file can not be converted to integer exit code".to_string(),
            });
        };
        Ok(ExitCode(exit_code))
    }

    /// Returns the expected stdout buffer for this command spec.
    pub fn expected_stdout(&self) -> Result<Vec<u8>, Error> {
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
