use regex::Match;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ReadState {
    WithoutPattern,
    WithPattern,
    Error,
    Eof,
}

pub struct PatternLines<'input> {
    chars: Peekable<Chars<'input>>,
    read_state: ReadState,
    line: String,
    pattern_start: String,
    pattern_end: String,
}

impl<'input> PatternLines<'input> {
    pub fn new(text: &'input str) -> Self {
        let chars = text.chars().peekable();
        let line = String::new();
        let pattern_start = "<<<".to_string();
        let pattern_end = ">>>".to_string();
        PatternLines {
            chars,
            read_state: ReadState::WithoutPattern,
            line,
            pattern_start,
            pattern_end,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternLine {
    NoPattern(String),
    Pattern(Regex),
}

/// This new type is necessary as `regex::Regex` doesn't implement `Eq` and `PartialEq`.
#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);

impl Regex {
    pub fn new(s: &str) -> Result<Self, regex::Error> {
        let re = regex::Regex::new(s)?;
        Ok(Regex(re))
    }

    pub fn find<'h>(&self, haystack: &'h str) -> Option<Match<'h>> {
        self.0.find(haystack)
    }
}

impl PartialEq for Regex {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl Eq for Regex {}

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Iterator for PatternLines<'_> {
    type Item = Result<PatternLine, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.read_state == ReadState::Error || self.read_state == ReadState::Eof {
            return None;
        }

        while let Some(&c) = self.chars.peek() {
            // Test if we have a start of a new pattern
            if self.is_pattern_start() {
                // Now, we're constructing a pattern
                self.read_state = ReadState::WithPattern;

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
                self.line.push(c);
            }

            // We test if we need to finish our chunk
            let new_line = c == '\n';
            let eof = self.chars.peek().is_none();
            if new_line || eof {
                let line = &self.line;
                let chunk = match self.read_state {
                    ReadState::WithoutPattern => PatternLine::NoPattern(line.clone()),
                    ReadState::WithPattern => {
                        let re = match Regex::new(line) {
                            Ok(re) => re,
                            Err(error) => {
                                self.read_state = ReadState::Error;
                                return Some(Err(error.to_string()));
                            }
                        };
                        PatternLine::Pattern(re)
                    }
                    _ => unreachable!(),
                };

                self.read_state = if eof {
                    ReadState::Eof
                } else {
                    // We restart from no patter, by default.
                    ReadState::WithoutPattern
                };
                self.line.clear();
                return Some(Ok(chunk));
            }
        }
        None
    }
}

impl PatternLines<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_chunk() {
        let input = "Hello <<<.*>>>!\nabcdef\n<<<[abcd]>>>foo bar baz<<<1234567891\\d>>>dummy";
        let mut lines = PatternLines::new(input);
        assert_eq!(
            lines.next(),
            Some(Ok(PatternLine::Pattern(Regex::new("Hello .*!\n").unwrap())))
        );
        assert_eq!(
            lines.next(),
            Some(Ok(PatternLine::NoPattern("abcdef\n".to_string())))
        );
        assert_eq!(
            lines.next(),
            Some(Ok(PatternLine::Pattern(
                Regex::new("[abcd]foo bar baz1234567891\\ddummy").unwrap()
            )))
        );
        assert_eq!(lines.next(), None)
    }

    #[test]
    fn test_invalid_pattern() {
        let input = "abcd\n<<< not end pattern";
        let mut lines = PatternLines::new(input);
        assert_eq!(
            lines.next(),
            Some(Ok(PatternLine::NoPattern("abcd\n".to_string())))
        );
        assert_eq!(lines.next(), Some(Err("pattern is invalid".to_string())));
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn test_invalid_regex() {
        let input = "<<<*>>>";
        let mut lines = PatternLines::new(input);
        let line = lines.next().unwrap();
        assert!(line.is_err());
    }
}
