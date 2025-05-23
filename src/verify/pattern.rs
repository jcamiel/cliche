use crate::chunk::{PatternLine, PatternLines};
use crate::verify::diff::{Diff, Error};

pub fn eval_pat_diff(expected: &str, actual: &[u8]) -> Result<Option<Diff>, Error> {
    // We accept lossy UTF-8 string for actual to detect encoding errors.
    let actual = String::from_utf8_lossy(actual).to_string();
    let mut actual_lines = actual.split_inclusive('\n');
    let expected_lines = PatternLines::new(expected);

    // We consume line pattern by line pattern and test each pattern. At the end, we must have
    // consume all the actual string, otherwise we have a mismatch.
    let mut row = 1;
    for expected_line in expected_lines {
        // Do we have a valid expected line?
        let expected_line = match expected_line {
            Err(error) => {
                return Err(Error::InvalidPattern {
                    reason: error.to_string(),
                    row,
                });
            }
            Ok(line) => line,
        };

        // No we test all the possible chunks variant.
        match expected_line {
            PatternLine::NoPattern(expected_line) => {
                // Do we have something in value to compare against?
                let Some(actual_line) = actual_lines.next() else {
                    let diff = Diff::Line {
                        expected: Some(expected_line),
                        actual: None,
                        row,
                    };
                    return Ok(Some(diff));
                };

                // We know that there is some actual value left
                if expected_line != actual_line {
                    let diff = Diff::Line {
                        expected: Some(expected_line),
                        actual: Some(actual_line.to_string()),
                        row,
                    };
                    return Ok(Some(diff));
                }
            }
            PatternLine::Pattern(expected_line) => {
                // Do we have something in value to compare against?
                let Some(actual_line) = actual_lines.next() else {
                    let diff = Diff::PatternLine {
                        expected: Some(expected_line.to_string()),
                        actual: None,
                        row,
                    };
                    return Ok(Some(diff));
                };

                let mat = expected_line.find(actual_line);
                match mat {
                    Some(mat) => {
                        // We have a match but not at the beginning of expected line
                        if mat.start() != 0 {
                            let diff = Diff::PatternLine {
                                expected: Some(expected_line.to_string()),
                                actual: Some(actual_line.to_string()),
                                row,
                            };
                            return Ok(Some(diff));
                        }
                    }
                    None => {
                        // We don't have any match
                        let diff = Diff::PatternLine {
                            expected: Some(expected_line.to_string()),
                            actual: Some(actual_line.to_string()),
                            row,
                        };
                        return Ok(Some(diff));
                    }
                }
            }
        }

        row += 1;
    }

    // We have consumed all the expected lines, do we have cosumed all the actual?
    if let Some(actual_line) = actual_lines.next() {
        let diff = Diff::Line {
            expected: None,
            actual: Some(actual_line.to_string()),
            row,
        };
        return Ok(Some(diff));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pat_none_diff() {
        // Diff with no pattern
        let expected = "foo\nbar\nbaz\n";
        let actual = "foo\nbar\nbaz\n".as_bytes();
        let diff = eval_pat_diff(expected, actual).unwrap();
        assert!(diff.is_none());

        // Diff with simple pattern
        let expected = "foo\n<<<.*>>>\nbaz\n";
        let actual = "foo\nbar\nbaz\n".as_bytes();
        let diff = eval_pat_diff(expected, actual).unwrap();
        assert!(diff.is_none());

        let expected = "foo\n<<<.*>>>\n<<<[ab]{2}>>>z\n";
        let actual = "foo\nbar\nbaz\n".as_bytes();
        let diff = eval_pat_diff(expected, actual).unwrap();
        assert!(diff.is_none());
    }

    #[test]
    fn test_pat_diff() {
        // Diff with a line diff
        let expected = "foo\nbar";
        let actual = "foo\nbaz".as_bytes();
        let diff = eval_pat_diff(expected, actual).unwrap();
        assert_eq!(
            diff,
            Some(Diff::Line {
                expected: Some("bar".to_string()),
                actual: Some("baz".to_string()),
                row: 2,
            })
        );

        // Diff with a non match pattern
        let expected = "foo\n<<<.*>>>\n<<<[ab]{2}>>>\n";
        let actual = "foo\nbar\nbaz\n".as_bytes();
        let diff = eval_pat_diff(expected, actual).unwrap();
        assert_eq!(
            diff,
            Some(Diff::PatternLine {
                expected: Some("[ab]{2}\n".to_string()),
                actual: Some("baz\n".to_string()),
                row: 3,
            })
        );
    }
}
