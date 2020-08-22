use thiserror::Error;

use crate::glyph::*;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("glyph reference {0:?} not present in provided GlyphOrder")]
    UnknownGlyphRef(GlyphRef)
}

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("tried to decode a {0}, but the buffer was too small")]
    BufferUnderflow(&'static str),

    #[error("invalid/unrecognised value {0} in {1}")]
    InvalidValue(&'static str, String)
}

pub type EncodeResult<T> = Result<T, EncodeError>;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("u16 Overflow ({scope}::{item} is {value})")]
    U16Overflow {
        scope: String,
        item: &'static str,
        value: usize
    },

    #[error("tried to encode a {0}, but the buffer was too small")]
    BufferTooSmallForType(&'static str)
}
