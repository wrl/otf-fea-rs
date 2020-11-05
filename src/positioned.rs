#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Positioned<T> {
    pub value: T,
    pub span: SourceSpan
}
