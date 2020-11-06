use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::coverage::*;


#[derive(Debug)]
pub struct SingleArray {
    glyphs: CoverageLookup<ValueRecord>,
    common_value_format: Option<u16>,
}

impl SingleArray {
    pub fn new() -> Self {
        Self {
            glyphs: CoverageLookup::new(),
            common_value_format: None
        }
    }

    pub fn new_with_value_format(format: u16) -> Self {
        Self {
            glyphs: CoverageLookup::new(),
            common_value_format: Some(format)
        }
    }

    pub fn add_glyph(&mut self, glyph: u16, value_record: ValueRecord) {
        self.glyphs.insert(glyph, value_record);
    }

    pub fn can_add(&self, glyph: u16, value_record: &ValueRecord) -> bool {
        // fonttools allows for duplicate rule definitions as long as the value record matches, but
        // since we don't want to create a many-rules-to-one-binary-value situation, we don't allow
        // duplicate keys at all. duplicate rules will have to go into a different subtable.
        //
        // we may want to consider rejecting this as invalid syntax/behaviour in the future.

        let vf_match = match self.common_value_format {
            Some(format) => value_record.smallest_possible_format() == format,
            _ => true
        };

        vf_match && !self.glyphs.get(&glyph).is_some()
    }
}

impl Default for SingleArray {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct SinglePosFormat2Header {
    pub format: u16,
    pub coverage_offset: u16,
    pub value_format: u16,
    pub value_count: u16
}

impl TTFEncode for SingleArray {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let value_format = match self.common_value_format {
            Some(format) => format,
            None => self.glyphs.values()
                        .map(|vr| vr.smallest_possible_format())
                        .fold(0u16, |vr, smallest| vr | smallest)
        };

        buf.defer_header_encode(
            |buf| Ok(SinglePosFormat2Header {
                format: 1,
                coverage_offset: (self.glyphs.ttf_encode(buf)? - start) as u16,
                value_format,
                value_count: self.glyphs.len() as u16
            }),

            |buf| {
                for vr in self.glyphs.values() {
                    vr.encode_to_format(buf, value_format, start)?;
                }

                Ok(())
            })
   }
}
