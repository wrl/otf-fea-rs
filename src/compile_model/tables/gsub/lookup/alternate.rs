use std::ops;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::coverage::*;


type inner = CoverageLookup<Vec<u16>>;

#[derive(Debug, Default)]
pub struct Alternate(pub inner);

impl ops::Deref for Alternate {
    type Target = inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Alternate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct AlternateSubstFormat1Header {
    format: u16,
    coverage_offset: u16,
    set_count: u16
}

impl TTFEncode for Alternate {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.bytes.resize(start + AlternateSubstFormat1Header::PACKED_LEN, 0u8);

        let record_start = buf.bytes.len();
        buf.bytes.resize(record_start + (u16::PACKED_LEN * self.len()), 0u8);

        buf.encode_pool_dedup(start, record_start, self.values(),
            |offset, _| offset,
            |buf, set| {
                buf.append(&(set.len() as u16))?;

                for glyph_id in set {
                    buf.append(glyph_id)?;
                }

                Ok(())
            })?;

        let header = AlternateSubstFormat1Header {
            format: 1,
            coverage_offset: (self.0.ttf_encode(buf)? - start) as u16,
            set_count: self.len() as u16
        };

        buf.encode_at(&header, start)
    }
}
