use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("u16 Overflow ({scope}::{item} is {value})")]
    U16Overflow {
        scope: String,
        item: &'static str,
        value: usize
    },

    #[error("tried to encode a {0}, but the buffer was too small")]
    BufferTooSmallForType(&'static str)
}
