use std::str::CharIndices;

/// An iterator at most `max_chunk_size` chars.
#[derive(Clone, Debug)]
pub struct ChunkedLines<'input> {
    chars: CharIndices<'input>,
    max_chunk_size: usize,
    input: &'input str,
    remaining: bool,
    current_row: u64,
    chunk_start_offset: usize,
}

impl<'input> ChunkedLines<'input> {
    pub fn new(text: &'input str, max_chunk_size: usize) -> Self {
        let chars = text.char_indices();
        let current_row = 1;
        let chunk_start_offset = 0;
        ChunkedLines {
            chars,
            max_chunk_size,
            input: text,
            remaining: true,
            current_row,
            chunk_start_offset,
        }
    }
}

pub struct ChunkedLine<'input> {
    value: &'input str,
    row: u64,
}

impl<'input> ChunkedLine<'input> {
    pub fn as_str(&self) -> &'input str {
        self.value
    }

    pub fn row(&self) -> u64 {
        self.row
    }
}

impl<'input> Iterator for ChunkedLines<'input> {
    type Item = ChunkedLine<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.remaining {
            return None;
        }

        while let Some((_, c)) = self.chars.next() {
            let next_offset = self.chars.offset();
            // If we've reached the maximum chunk size or if we have a newline, we return
            // the new chunk.
            let new_line = c == '\n';
            let line = &self.input[self.chunk_start_offset..next_offset];
            let len = line.len();
            if len >= self.max_chunk_size || new_line {
                let row = self.current_row;
                self.chunk_start_offset = next_offset;
                if new_line {
                    self.current_row += 1;
                }
                let chunk = ChunkedLine { value: line, row };
                return Some(chunk);
            }
        }

        // We're at the end of the string
        let final_line = &self.input[self.chunk_start_offset..];
        self.chunk_start_offset = self.chars.offset();
        self.remaining = false;
        if final_line.is_empty() {
            None
        } else {
            let chunk = ChunkedLine {
                value: final_line,
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
        let chunks = ChunkedLines::new(input, max_chunk_size)
            .map(|chunk| chunk.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            &chunks[..],
            [
                "This is a short line.\n",
                "This line is exactly 64 characters long (include newline) 12345\n",
                "This line is way too long and needs to be split multiple times i",
                "nto chunks of 64 characters or fewer."
            ]
        )
    }

    #[test]
    fn chunk_at_multi_bytes_chars() {
        let input = "🍌🍓🍉🍇🥝🍒🍋🍏🍍🍐🥭🍊";
        let max_chunk_size = 1;
        let chunks = ChunkedLines::new(input, max_chunk_size)
            .map(|chunk| chunk.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            &chunks[..],
            [
                "🍌", "🍓", "🍉", "🍇", "🥝", "🍒", "🍋", "🍏", "🍍", "🍐", "🥭", "🍊"
            ],
        )
    }
}
