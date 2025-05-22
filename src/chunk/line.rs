use std::str::CharIndices;

pub struct ChunkedLines<'input> {
    chars: CharIndices<'input>,
    max_chunk_size: usize,
    offset: usize,
    input: &'input str,
    remaining: bool,
    current_row: u64,
}

pub struct ChunkedLine<'input> {
    line: &'input str,
    row: u64,
}

impl<'input> ChunkedLine<'input> {
    pub fn as_str(&self) -> &'input str {
        self.line
    }

    pub fn row(&self) -> u64 {
        self.row
    }
}

impl<'input> ChunkedLines<'input> {
    pub fn new(text: &'input str, max_chunk_size: usize) -> Self {
        let chars = text.char_indices();
        let offset = 0;
        let remaining = true;
        let current_row = 1;
        ChunkedLines {
            chars,
            max_chunk_size,
            offset,
            input: text,
            remaining,
            current_row,
        }
    }
}

impl<'input> Iterator for ChunkedLines<'input> {
    type Item = ChunkedLine<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.remaining {
            return None;
        }
        while let Some((i, c)) = self.chars.next() {
            let len = i - self.offset + 1;
            // If we've reached the maximum chunk size or if we have a newline, we return
            // the new chunk.
            let new_line = c == '\n';
            if len >= self.max_chunk_size || new_line {
                let row = self.current_row;
                let line = &self.input[self.offset..=i];
                self.offset = i + 1;
                if new_line {
                    self.current_row += 1;
                }
                let chunk = ChunkedLine { line, row };
                return Some(chunk);
            }
        }

        // We're at the end of the string
        let final_line = &self.input[self.offset..];
        self.remaining = false;
        if final_line.is_empty() {
            None
        } else {
            let chunk = ChunkedLine {
                line: final_line,
                row: self.current_row,
            };
            Some(chunk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_at_64_chars() {
        let input = "This is a short line.\nThis line is exactly 64 characters long (include newline) 12345\nThis line is way too long and needs to be split multiple times into chunks of 64 characters or fewer.";
        let max_chunk_size = 64;

        for chunk in ChunkedLines::new(input, max_chunk_size) {
            println!("Chunk: '{}'", chunk.as_str());
        }
    }
}
