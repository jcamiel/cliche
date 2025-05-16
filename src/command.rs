use crate::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

pub struct CommandSpec {
    cmd_path: PathBuf,
    stdout_path: PathBuf,
    /// Expected exit code of the command.
    exit_code: i32,
}

impl CommandSpec {
    pub fn new(cmd: &Path, exit_code: i32) -> Self {
        let mut stdout_path = PathBuf::from(cmd);
        stdout_path.set_extension("out");
        CommandSpec {
            cmd_path: cmd.to_path_buf(),
            stdout_path,
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
        let stdout_path = match fs::canonicalize(&self.stdout_path) {
            Ok(s) => s,
            Err(_) => return Ok(())
        };

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
