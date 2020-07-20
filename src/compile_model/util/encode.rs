use endian_codec::EncodeBE;

pub use crate::compile_model::error::{
    EncodeError,
    EncodeResult
};

pub struct EncodeBuf {
    pub(crate) bytes: Vec<u8>
}

impl EncodeBuf {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new()
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &*self.bytes
    }

    #[inline]
    pub(crate) fn append<T: TTFEncode>(&mut self, val: &T) -> EncodeResult<usize> {
        val.ttf_encode(self)
    }

    #[inline]
    pub(crate) fn encode_at<T: EncodeBE>(&mut self, val: &T, start: usize)
            -> EncodeResult<usize> {
        let end = start + T::PACKED_LEN;

        if end > self.bytes.len() {
            // FIXME: does this correctly stringify the type name,
            // or do we just get a string of "T"?
            return Err(EncodeError::BufferTooSmallForType(stringify!(T)));
        }

        val.encode_as_be_bytes(&mut self.bytes[start..end]);
        Ok(start)
    }
}

pub trait TTFEncode: Sized {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize>;
}

impl<T: EncodeBE> TTFEncode for T
{
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let end = start + T::PACKED_LEN;

        buf.bytes.resize(end, 0u8);
        self.encode_as_be_bytes(&mut buf.bytes[start..end]);

        Ok(start)
    }
}
