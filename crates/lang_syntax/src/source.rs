#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourcePosition {
    pub byte: usize,
    pub line: usize,
    pub column: usize,
}

impl SourcePosition {
    pub const fn start() -> Self {
        Self {
            byte: 0,
            line: 1,
            column: 1,
        }
    }
}
