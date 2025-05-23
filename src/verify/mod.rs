use crate::command::{CommandResult, CommandSpec};
use crate::error::Error;
use crate::verify::diff::Diff;

mod diff;
mod exact;
mod pattern;

pub fn check_result(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    check_exit_code(cmd, result)?;

    // Possible cases:
    // - only `foo.out` exists: we check the expected stdout against the actual stdout,
    // - only `foo.out.pattern` exists: we check the expected pattern against the actual stdout,
    // - `foo.out.pattern` and `foo.out` exist: we both check the expected pattern and the expected
    // stdout against the actual stdout
    // - neither `foo.out.pattern` nor `foo.out` exist: we chgeck that actual stdout is empty.

    if cmd.has_stdout() && cmd.has_stdout_pat() {
        check_equal_stdout(cmd, result)?;
        check_equal_stdout_pat(cmd, result)?;
    } else if cmd.has_stdout() {
        check_equal_stdout(cmd, result)?;
    } else if cmd.has_stdout_pat() {
        check_equal_stdout_pat(cmd, result)?;
    } else {
        check_empty_stdout(cmd, result)?;
    }

    // We apply the same check for stderr:
    if cmd.has_stderr() {
        check_equal_stderr(cmd, result)?;
    }

    Ok(())
}

/// Check the exit code of the `cmd` against a `result` exit code.
fn check_exit_code(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    let expected_exit_code = cmd.exit_code()?;
    let actual_exit_code = result.exit_code();
    if expected_exit_code != actual_exit_code {
        let err = Error::CheckExitCode {
            cmd_path: cmd.cmd_path().to_path_buf(),
            expected: expected_exit_code,
            actual: actual_exit_code,
            stderr: result.stderr().to_vec(),
        };
        return Err(err);
    }
    Ok(())
}

fn check_equal_stdout(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    let expected = cmd.stdout()?;
    let actual = result.stdout().to_vec();

    let diff = exact::eval_exact_diff(&expected, &actual);
    match diff {
        None => Ok(()),
        Some(Diff::Line {
            expected,
            actual,
            row,
        }) => Err(Error::CheckStdoutLine {
            cmd_path: cmd.cmd_path().to_path_buf(),
            expected,
            actual,
            row,
        }),
        Some(Diff::Byte) => todo!(),
        Some(Diff::PatternLine { .. }) => unreachable!(),
    }
}

fn check_equal_stderr(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    let expected = cmd.stderr()?;
    let actual = result.stderr().to_vec();

    let diff = exact::eval_exact_diff(&expected, &actual);
    match diff {
        None => Ok(()),
        Some(Diff::Line {
            expected,
            actual,
            row,
        }) => Err(Error::CheckStderrLine {
            cmd_path: cmd.cmd_path().to_path_buf(),
            expected,
            actual,
            row,
        }),
        Some(Diff::Byte) => todo!(),
        Some(Diff::PatternLine { .. }) => unreachable!(),
    }
}

fn check_equal_stdout_pat(cmd: &CommandSpec, result: &CommandResult) -> Result<(), Error> {
    let expected_stdout_pat = cmd.stdout_pat()?;
    let actual_stdout = result.stdout().to_vec();
    let diff = pattern::eval_pat_diff(&expected_stdout_pat, &actual_stdout);
    let diff = match diff {
        Ok(d) => d,
        Err(diff::Error::InvalidPattern { reason, row }) => {
            return Err(Error::StdoutPatternFileInvalid {
                cmd_path: cmd.cmd_path().to_path_buf(),
                reason,
                row,
            });
        }
    };

    match diff {
        None => Ok(()),
        Some(Diff::Line {
            expected,
            actual,
            row,
        }) => Err(Error::CheckStdoutLine {
            cmd_path: cmd.cmd_path().to_path_buf(),
            expected,
            actual,
            row,
        }),
        Some(Diff::Byte) => unreachable!(),
        Some(Diff::PatternLine {
            expected,
            actual,
            row,
        }) => Err(Error::CheckStdoutPattern {
            cmd_path: cmd.cmd_path().to_path_buf(),
            expected,
            actual,
            row,
        }),
    }
}

// TODO:
fn check_empty_stdout(_cmd: &CommandSpec, _result: &CommandResult) -> Result<(), Error> {
    Ok(())
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
