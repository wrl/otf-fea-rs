use endian_codec::EncodeBE;

#[inline]
pub(crate) fn encode_u16_be(bytes: &mut [u8], offset: usize, val: u16) {
    &bytes[offset..offset+2].copy_from_slice(&val.to_be_bytes());
}

#[inline]
pub fn encode_to_slice<T: EncodeBE>(bytes: &mut [u8], val: &T) {
    val.encode_as_be_bytes(&mut bytes[..T::PACKED_LEN])
}

pub(crate) trait TTFEncodeExt<'a, T>: Iterator<Item = &'a T>
    where T: EncodeBE + 'a,
          Self: Sized + 'a
{
    #[inline]
    fn collect_into_ttf_pool(self, buf: &mut Vec<u8>) {
        for item in self {
            let start = buf.len();
            let end = start + T::PACKED_LEN;
            buf.resize(end, 0u8);

            encode_to_slice(&mut buf[start..end], item);
        }
    }
}

impl<'a, T, I> TTFEncodeExt<'a, T> for I
    where T: EncodeBE + 'a,
          I: Iterator<Item = &'a T> + Sized + 'a
{ }
