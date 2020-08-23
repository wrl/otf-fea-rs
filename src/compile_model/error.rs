use thiserror::Error;

use crate::glyph_order::*;
use crate::tag::*;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    GlyphOrderError(#[from] GlyphOrderError)
}

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("tried to decode a {0}, but the buffer was too small")]
    BufferUnderflow(&'static str),

    #[error("{0} referenced feature at index {1}, which does not exist")]
    UndefinedFeature(&'static str, u16),

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

    #[error("{0} referenced tag {1}, which is not in the feature list")]
    TagNotInFeatureList(&'static str, Tag),

    #[error("tried to encode a {0}, but the buffer was too small")]
    BufferTooSmallForType(&'static str)
}
