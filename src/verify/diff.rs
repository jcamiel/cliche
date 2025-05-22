use crate::chunk::ChunkedLines;
use std::cmp::max;
use std::str;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diff {
    Line {
        expected: Option<String>,
        actual: Option<String>,
        number: u64,
    },
    Byte,
}

pub fn eval_diff(expected: &[u8], actual: &[u8]) -> Option<Diff> {
    // If we can convert actual and expected stdout to text, we split them to line chunks
    // and we compare them line by line.
    // We accept to have lossy UTF_8 conversion for actual string, but we expect valid UTF-8 string on
    // expected.
    let expected_str = str::from_utf8(&expected);
    let actual_str = String::from_utf8_lossy(&actual);
    match (expected_str, actual_str) {
        (Ok(expected), actual) => {
            // Two stdouts are UTF-8 valid (actual can have replacement chars `U+FFFD REPLACEMENT CHARACTER`)
            // we're comparing then by chunks of max 64 chars. The chunks can split if there are
            // newlines.
            eval_diff_as_str(expected, actual.as_ref())
        }
        _ => {
            // One of the stdout is not a valid UTF_8 string, we make a byte to byte comparison.
            eval_diff_as_bytes(&expected, &actual)
        }
    }
}

/// Returns the first line difference between an `expected` string and an `actual` string.
fn eval_diff_as_str(expected: &str, actual: &str) -> Option<Diff> {
    let max_chunk_size = 64;
    let expected = ChunkedLines::new(&expected, max_chunk_size).collect::<Vec<_>>();
    let actual = ChunkedLines::new(&actual, max_chunk_size).collect::<Vec<_>>();
    let max_chunks = max(actual.len(), expected.len());
    for i in 0..max_chunks {
        let actual_chunk = actual.get(i);
        let expected_chunk = expected.get(i);
        match (actual_chunk, expected_chunk) {
            // On the same line, two stdout differs
            (Some(actual_chunk), Some(expected_chunk)) => {
                let actual = actual_chunk.as_str();
                let expected = expected_chunk.as_str();
                let row = actual_chunk.row();
                if actual == expected {
                    continue;
                } else {
                    let diff = Diff::Line {
                        expected: Some(expected.to_string()),
                        actual: Some(actual.to_string()),
                        number: row,
                    };
                    return Some(diff);
                }
            }
            // There are more actual lines that expected lines
            (Some(actual_chunk), None) => {
                let actual = actual_chunk.as_str().to_string();
                let row = actual_chunk.row();
                let diff = Diff::Line {
                    expected: None,
                    actual: Some(actual),
                    number: row,
                };
                return Some(diff);
            }
            // There are less actual lines that expected lines
            (None, Some(expected_chunk)) => {
                let expected = expected_chunk.as_str().to_string();
                let row = expected_chunk.row();
                let diff = Diff::Line {
                    expected: Some(expected),
                    actual: None,
                    number: row,
                };
                return Some(diff);
            }
            // End of diff, everyting is good
            (None, None) => {
                break;
            }
        }
    }
    None
}

/// Returns the first byte difference between an `expected` string and an `actual` string.
fn eval_diff_as_bytes(_expected: &[u8], _actual: &[u8]) -> Option<Diff> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_diff_with_bad_encoding() {
        // Café in latin 1
        let actual = [0x63, 0x61, 0x66, 0xe9];
        let expected = [0x63, 0x61, 0x66, 0xc3, 0xa9];
        let diff = eval_diff(&expected, &actual).unwrap();
        assert_eq!(
            diff,
            Diff::Line {
                expected: Some("café".to_string()),
                actual: Some("caf�".to_string()),
                number: 1
            }
        );
    }


    #[test]
    fn test_diff_as_str() {
        let expected = "foo\nbar\nbaz\n";
        let actual = "foo\nbar\nbaz\n";
        assert!(eval_diff_as_str(expected, actual).is_none());

        let expected = "aaaa\nbbbb\ncccc\n";
        let actual = "aaaa\nbbbb\ncc-c\n";
        let diff = eval_diff_as_str(expected, actual).unwrap();
        assert_eq!(
            diff,
            Diff::Line {
                expected: Some("cccc\n".to_string()),
                actual: Some("cc-c\n".to_string()),
                number: 3
            }
        );

        // More actual lines than expected
        let expected = "aaaa\nbbbb\ncccc\n";
        let actual = "aaaa\nbbbb\ncccc\ndddd\n";
        let diff = eval_diff_as_str(expected, actual).unwrap();
        assert_eq!(
            diff,
            Diff::Line {
                expected: None,
                actual: Some("dddd\n".to_string()),
                number: 4
            }
        );

        // A very long line
        let expected = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis xxx nostrud exercitation ullamco laboris";
        let actual = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris";
        let diff = eval_diff_as_str(expected, actual).unwrap();
        assert_eq!(
            diff,
            Diff::Line {
                expected: Some(
                    "nim ad minim veniam, quis xxx nostrud exercitation ullamco labor".to_string()
                ),
                actual: Some(
                    "nim ad minim veniam, quis nostrud exercitation ullamco laboris".to_string()
                ),
                number: 1
            }
        );
    }
}
