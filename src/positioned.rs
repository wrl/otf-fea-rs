use std::ops::Deref;
use std::fmt;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize
}

impl fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} - {}:{}",
            self.start.line, self.start.column,
            self.end.line, self.end.column)
    }
}

/// A type representing `T` with position information from the source file.
///
/// This type is used in the parser to "wrap" arbitrary other types and indicate the SourceSpan
/// from whence the type was parsed.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Positioned<T> {
    pub value: T,
    pub span: SourceSpan
}

impl<T> Deref for Positioned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}



/// A type representing `T` with possible source position information.
///
/// This type is used in the compiler to indicate that a value can come from a parsed value, but
/// does not necessarily have to (for example, when loading a binary font file into the
/// compiler_model representations).

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MaybePositioned<T> {
    pub value: T,
    pub span: Option<SourceSpan>
}

impl<T: fmt::Display> fmt::Display for MaybePositioned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{} ({})", self.value, span.start)
        } else {
            self.value.fmt(f)
        }
    }
}

impl<T> MaybePositioned<T> {
    #[inline]
    pub fn has_position(&self) -> bool {
        self.span.is_some()
    }
}

impl<T> From<Positioned<T>> for MaybePositioned<T> {
    fn from(other: Positioned<T>) -> Self {
        Self {
            value: other.value,
            span: Some(other.span)
        }
    }
}

impl<T> From<T> for MaybePositioned<T> {
    fn from(value: T) -> Self {
        Self::unpositioned(value)
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

impl<T> Deref for MaybePositioned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
