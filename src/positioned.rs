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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MaybePositioned<T> {
    pub value: T,
    pub span: Option<SourceSpan>
}

impl<T> From<Positioned<T>> for MaybePositioned<T> {
    fn from(other: Positioned<T>) -> Self {
        Self {
            value: other.value,
            span: Some(other.span)
        }
    }
}

impl<T> MaybePositioned<T> {
    pub fn unpositioned(value: T) -> Self {
        Self {
            value,
            span: None
        }
    }
}
