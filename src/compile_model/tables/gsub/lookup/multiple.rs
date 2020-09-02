use std::ops;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::coverage::*;


type inner = CoverageLookup<Vec<u16>>;

#[derive(Debug, Default)]
pub struct Multiple(pub inner);

impl ops::Deref for Multiple {
    type Target = inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Multiple {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct MultipleSubstFormat1Header {
    format: u16,
    coverage_offset: u16,
    sequence_count: u16
}


impl TTFEncode for Multiple {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.bytes.resize(start + MultipleSubstFormat1Header::PACKED_LEN, 0u8);

        let record_start = buf.bytes.len();
        buf.bytes.resize(record_start + (u16::PACKED_LEN * self.len()), 0u8);

        buf.encode_pool_dedup(start, record_start, self.values(),
            |offset, _| offset,
            |buf, seq| {
                buf.append(&(seq.len() as u16))?;

                for glyph_id in seq {
                    buf.append(glyph_id)?;
                }

                Ok(())
            })?;

        let header = MultipleSubstFormat1Header {
            format: 1,
            coverage_offset: (self.0.ttf_encode(buf)? - start) as u16,
            sequence_count: self.len() as u16
        };

        buf.encode_at(&header, start)
    }
}
