use std::iter::Peekable;
use std::str::Chars;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ReadState {
    SimpleLine,
    HasPattern,
    Error,
    Eof,
}

pub struct ChunkedPatterns<'input> {
    chars: Peekable<Chars<'input>>,
    max_chunk_size: usize,
    current_row: u64,
    read_state: ReadState,
    line: String,
    pattern_start: String,
    pattern_end: String,
}

impl<'input> ChunkedPatterns<'input> {
    pub fn new(text: &'input str, max_chunk_size: usize) -> Self {
        let chars = text.chars().peekable();
        let current_row = 1;
        let line = String::with_capacity(max_chunk_size);
        let pattern_start = "<<<".to_string();
        let pattern_end = ">>>".to_string();

        ChunkedPatterns {
            chars,
            max_chunk_size,
            current_row,
            read_state: ReadState::SimpleLine,
            line,
            pattern_start,
            pattern_end,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChunkPattern {
    SimpleLine {
        value: String,
        row: u64,
    },
    PatternLine {
        value: String,
        // todo compute the regex here
        // TODO: keep the source
        // regex: String,
        row: u64,
    },
}

impl Iterator for ChunkedPatterns<'_> {
    type Item = Result<ChunkPattern, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.read_state == ReadState::Error || self.read_state == ReadState::Eof {
            return None;
        }

        while let Some(&c) = self.chars.peek() {
            // Test if we have a start of a new pattern
            if self.is_pattern_start() {
                // Now, we're constructing a pattern
                self.read_state = ReadState::HasPattern;

                // We read the regex inside the pattern
                let pat = self.read_pattern();
                let pat = match pat {
                    Ok(p) => p,
                    Err(_) => {
                        self.read_state = ReadState::Error;
                        return Some(Err("pattern is invalid".to_string()));
                    }
                };
                self.line.push_str(&pat);
            } else {
                self.chars.next();
                let escaped = escape_regex_metacharacter(c);
                self.line.push_str(&escaped);
            }

            // We test if we need to finish our chunk
            let new_line = c == '\n';
            let eof = self.chars.peek().is_none();
            let len = self.line.len();
            if len >= self.max_chunk_size || new_line || eof {
                let row = self.current_row;
                let line = self.line.clone();
                let chunk = match self.read_state {
                    ReadState::SimpleLine => ChunkPattern::SimpleLine { value: line, row },
                    ReadState::HasPattern => ChunkPattern::PatternLine { value: line, row },
                    _ => unreachable!(),
                };

                if new_line {
                    self.current_row += 1;
                }
                self.read_state = if eof {
                    ReadState::Eof
                } else {
                    ReadState::SimpleLine
                };
                self.line.clear();
                return Some(Ok(chunk));
            }
        }
        None
    }
}

impl ChunkedPatterns<'_> {
    fn peek_n(&self, n: usize) -> String {
        // Clone our iterator, so we can read
        let next_chars = self.chars.clone();
        next_chars.take(n).collect::<String>()
    }

    fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.chars.next();
        }
    }

    fn is_pattern_start(&self) -> bool {
        let next = self.peek_n(self.pattern_start.len());
        next == self.pattern_start
    }

    fn skip_pattern_start(&mut self) {
        self.skip_n(self.pattern_start.len());
    }

    fn is_pattern_end(&self) -> bool {
        let next = self.peek_n(self.pattern_end.len());
        next == self.pattern_end
    }

    fn skip_pattern_end(&mut self) {
        self.skip_n(self.pattern_end.len());
    }

    fn read_pattern(&mut self) -> Result<String, ()> {
        let mut pattern = String::new();

        self.skip_pattern_start();
        while !self.is_pattern_end() {
            let next = self.chars.next();
            match next {
                None => {
                    // We have ended the text input chars while still in the pattern, it's
                    // an invalid patterned
                    return Err(());
                }
                Some(c) => pattern.push(c),
            }
        }
        self.skip_pattern_end();
        Ok(pattern)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_chunk() {
        let max_chunk_size = 32;
        let input = "Hello <<<.*>>>!\nabcdef\n<<<[abcd]>>>foo bar baz<<<1234567891\\d>>>dummy";
        let mut chunks = ChunkedPatterns::new(input, max_chunk_size);
        assert_eq!(
            chunks.next(),
            Some(Ok(ChunkPattern::PatternLine {
                value: "Hello .*!\n".to_string(),
                row: 1
            }))
        );
        assert_eq!(
            chunks.next(),
            Some(Ok(ChunkPattern::SimpleLine {
                value: "abcdef\n".to_string(),
                row: 2
            }))
        );
        assert_eq!(
            chunks.next(),
            Some(Ok(ChunkPattern::PatternLine {
                value: "[abcd]foo bar baz1234567891\\ddum".to_string(),
                row: 3
            }))
        );
        assert_eq!(
            chunks.next(),
            Some(Ok(ChunkPattern::SimpleLine {
                value: "my".to_string(),
                row: 3
            }))
        );
        assert_eq!(chunks.next(), None)
    }

    #[test]
    fn test_invalid_pattern() {
        let max_chunk_size = 32;
        let input = "abcd\n<<< not end pattern";
        let mut chunks = ChunkedPatterns::new(input, max_chunk_size);
        assert_eq!(
            chunks.next(),
            Some(Ok(ChunkPattern::SimpleLine {
                value: "abcd\n".to_string(),
                row: 1
            }))
        );
        assert_eq!(chunks.next(), Some(Err("pattern is invalid".to_string())));
        assert_eq!(chunks.next(), None);
    }
}
