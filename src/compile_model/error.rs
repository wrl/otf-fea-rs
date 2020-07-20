use thiserror::Error;

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
