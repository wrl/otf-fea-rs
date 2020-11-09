use thiserror::Error;

use crate::compile_model::tables::gpos::lookup::*;
use crate::glyph_order::*;
use crate::FeatureTag;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("{ty} Overflow ({scope}::{item} is {value})")]
    Overflow {
        ty: &'static str,
        scope: String,
        item: &'static str,
        value: isize
    },

    #[error("mark classes cannot be defined or amended after first reference in a position statement")]
    MarkClassNotAllowed,

    #[error("unknown mark class \"{0}\"")]
    UnknownMarkClass(String),

    #[error("unknown glyph class \"{0}\"")]
    UnknownGlyphClass(String),

    #[error("tried to compile an invalid anchor type {0}")]
    InvalidAnchor(&'static str),

    #[error("undefined {0} {1}")]
    UndefinedReference(&'static str, String),

    #[error(transparent)]
    GlyphOrderError(#[from] GlyphOrderError),

    #[error(transparent)]
    PairClassError(#[from] PairClassError)
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

    #[error("value record format includes fields which are not presetn")]
    ValueRecordFormatMismatch,

    #[error("{0} referenced tag {1}, which is not in the feature list")]
    TagNotInFeatureList(&'static str, FeatureTag),

    #[error("tried to encode a {0}, but the buffer was too small")]
    BufferTooSmallForType(&'static str)
}
