use endian_codec::PackedSize;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;


#[derive(Debug)]
pub struct LookupList<T>(pub Vec<T>);

impl<T> LookupList<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl<T: TTFDecode> TTFDecode for LookupList<T> {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let records_count = decode_u16_be(bytes, 0);
        let records = decode_from_pool(records_count, &bytes[2..]);

        let lookups = records
            .map(|offset: u16| T::ttf_decode(&bytes[offset as usize..]));

        lookups.collect::<DecodeResult<_>>()
            .map(Self)
    }
}

impl<T: TTFEncode> TTFEncode for LookupList<T> {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.append(&(self.0.len() as u16))?;
        let mut record_offset = buf.bytes.len();
        buf.bytes.resize(record_offset +
            (self.0.len() * u16::PACKED_LEN), 0u8);

        for lookup in &self.0 {
            let offset = (buf.append(lookup)? - start) as u16;

            buf.encode_at(&offset, record_offset)?;
            record_offset += u16::PACKED_LEN;
        }

        Ok(start)
    }
}
