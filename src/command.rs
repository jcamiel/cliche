use crate::command::VerifyError::ExitCode;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

pub struct CommandSpec {
    cmd: PathBuf,
    exit_code: i32,
}

pub enum VerifyError {
    ExitCode { expected: i32, actual: i32 },
}

impl CommandSpec {
    pub fn new(cmd: &Path, exit_code: i32) -> Self {
        CommandSpec {
            cmd: cmd.to_path_buf(),
            exit_code,
        }
    }

    pub fn execute(&self) -> Result<CommandResult, io::Error> {
        let cmd = fs::canonicalize(&self.cmd)?;
        let output = Command::new(cmd.as_os_str()).output()?;

        let exit_code = output.status.code().unwrap();
        let stdout = &output.stdout;
        let stderr = &output.stderr;
        Ok(CommandResult::new(exit_code, stdout, stderr))
    }

    pub fn verify(&self, result: &CommandResult) -> Result<(), VerifyError> {
        if self.exit_code != result.exit_code {
            let err = ExitCode {
                expected: self.exit_code,
                actual: result.exit_code,
            };
            return Err(err);
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
