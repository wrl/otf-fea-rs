use endian_codec::DecodeBE;

pub use crate::compile_model::error::{
    DecodeError,
    DecodeResult
};

pub trait TTFDecode: Sized {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self>;
}

#[inline]
pub(crate) fn decode_u16_be(bytes: &[u8], offset: usize) -> u16 {
    let mut a = [0u8; 2];
    a.copy_from_slice(&bytes[offset..offset+2]);
    u16::from_be_bytes(a)
}

#[inline]
pub fn decode_from_slice<T: DecodeBE>(bytes: &[u8]) -> T {
    T::decode_from_be_bytes(&bytes[..T::PACKED_LEN])
}

#[inline]
pub fn decode_from_pool<'a, T: DecodeBE>(count: u16, bytes: &'a [u8])
        -> impl Iterator<Item = T> + 'a
{
    (0..count)
        .map(move |i| {
            let start = i as usize * T::PACKED_LEN;
            decode_from_slice(&bytes[start..])
        })
}
