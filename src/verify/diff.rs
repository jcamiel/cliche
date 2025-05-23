#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Diff {
    Line {
        expected: Option<String>,
        actual: Option<String>,
        row: usize,
    },
    PatternLine {
        expected: Option<String>,
        actual: Option<String>,
        row: usize,
    },
    Byte,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidPattern { reason: String, row: usize },
}
