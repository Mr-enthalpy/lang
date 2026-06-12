/// Source span with byte offsets and 1-based source position.
///
/// v0.1 lexing normalizes CRLF and CR to LF before tokenization. `byte_start`
/// and `byte_end` are offsets into that normalized LF source text, not
/// necessarily the raw file bytes as checked out on disk. `byte_start` is
/// inclusive and `byte_end` is exclusive. `line` and `column` are computed over
/// normalized LF text, and `column` is a 1-based byte column for v0.1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub const fn new(byte_start: usize, byte_end: usize, line: usize, column: usize) -> Self {
        Self {
            byte_start,
            byte_end,
            line,
            column,
        }
    }

    pub const fn at(byte: usize, line: usize, column: usize) -> Self {
        Self::new(byte, byte, line, column)
    }

    pub fn join(self, other: Self) -> Self {
        Self::new(self.byte_start, other.byte_end, self.line, self.column)
    }
}
