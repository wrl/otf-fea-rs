use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::coverage::*;


#[derive(Debug)]
pub struct SingleClass {
    glyphs: CoverageLookup<()>,
    value_record: ValueRecord
}

impl SingleClass {
    pub fn new(value_record: ValueRecord) -> Self {
        Self {
            glyphs: CoverageLookup::new(),
            value_record,
        }
    }

    pub fn add_glyph(&mut self, glyph: u16) {
        self.glyphs.insert(glyph, ());
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct SinglePosFormat1Header {
    pub format: u16,
    pub coverage_offset: u16,
    pub value_format: u16
}

impl TTFEncode for SingleClass {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let value_format = self.value_record.smallest_possible_format();

        buf.defer_header_encode(
            |buf| Ok(SinglePosFormat1Header {
                format: 1,
                coverage_offset: (self.glyphs.ttf_encode(buf)? - start) as u16,
                value_format
            }),

            |buf| {
                self.value_record.encode_to_format(buf, value_format, start)
            })?;

        Ok(start)
   }
}
