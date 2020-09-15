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

        buf.defer_header_encode(
            |buf| Ok(AlternateSubstFormat1Header {
                format: 1,
                coverage_offset: (self.0.ttf_encode(buf)? - start) as u16,
                set_count: self.len() as u16
            }),

            |buf| buf.encode_pool(start, self.values(),
                |offset, _| offset,
                |buf, &set| {
                    buf.append(&(set.len() as u16))?;

                    for glyph_id in set {
                        buf.append(glyph_id)?;
                    }

                    Ok(())
                }))
    }
}
