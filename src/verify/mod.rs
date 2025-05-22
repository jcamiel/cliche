use crate::command::{CommandResult, CommandSpec};
use crate::error::Error;
use crate::verify::diff::Diff;

mod diff;

pub fn check_result(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    check_exit_code(cmd, result)?;
    check_stdout(cmd, result)?;
    Ok(())
}

/// Check the exit code of the `cmd` against a `result` exit code.
fn check_exit_code(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    let expected_exit_code = cmd.expected_exit_code()?;
    let actual_exit_code = result.exit_code();
    if expected_exit_code != actual_exit_code {
        let err = Error::ExitCodeCheck {
            expected: expected_exit_code,
            actual: actual_exit_code,
            stderr: result.stderr().to_vec(),
        };
        return Err(err);
    }
    Ok(())
}

fn check_stdout(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    let expected_stdout = cmd.expected_stdout()?;
    let actual_stdout = result.stdout().to_vec();

    let diff = diff::eval_diff(&expected_stdout, &actual_stdout);
    match diff {
        None => Ok(()),
        Some(Diff::Line {
            expected,
            actual,
            number: row,
        }) => Err(Error::StdoutLineCheck {
            cmd_path: cmd.cmd_path().to_path_buf(),
            expected,
            actual,
            line: row,
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CommandResult, CommandSpec};
    use std::fs::File;
    use std::io;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn write_file_with(dir: &Path, name: &str, text: &str) -> Result<PathBuf, io::Error> {
        let file_path = dir.join(name);
        let mut file = File::create(file_path.clone())?;
        writeln!(file, "{}", text)?;
        Ok(file_path)
    }

    #[test]
    fn test_no_expected_exit_code() {
        let tmp_dir = TempDir::new().unwrap();
        let cmd_path = write_file_with(tmp_dir.path(), "foo.sh", "echo 'Hello'").unwrap();

        let cmd = CommandSpec::new(&cmd_path).unwrap();
        let res = CommandResult::new(0.into(), &[], &[]);
        assert!(check_result(&cmd, &res).is_ok())
    }
}
