use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::coverage::*;


#[derive(Debug)]
pub struct Single {
    pub glyphs: CoverageLookup<ValueRecord>
}

impl Single {
    pub fn new() -> Self {
        Self {
            glyphs: CoverageLookup::new()
        }
    }

    pub fn add_glyph(&mut self, glyph: u16, value_record: ValueRecord) {
        self.glyphs.insert(glyph, value_record);
    }
}

impl Default for Single {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct SinglePosFormat1Header {
    pub format: u16,
    pub coverage_offset: u16,
    pub value_format: u16
}

impl TTFEncode for Single {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let value_format = self.glyphs.values()
            .map(|vr| vr.smallest_possible_format())
            .fold(0u16, |vr, smallest| vr | smallest);

        buf.bytes.resize(start + SinglePosFormat1Header::PACKED_LEN, 0u8);

        for vr in self.glyphs.values() {
            vr.encode_to_format(buf, value_format)?;
        }

        let header = SinglePosFormat1Header {
            format: 1,
            coverage_offset: (self.glyphs.ttf_encode(buf)? - start) as u16,
            value_format
        };

        buf.encode_at(&header, start)
   }
}
