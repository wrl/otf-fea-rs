use std::ops;
use std::collections::BTreeSet;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::glyph_class::*;
use crate::glyph_order::*;

use crate::compile_model::util::encode::*;

use crate::util::*;


#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ClassDef(pub BTreeSet<u16>);


impl Default for ClassDef {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl ops::Deref for ClassDef {
    type Target = BTreeSet<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for ClassDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
struct Format1Header {
    format: u16,
    start_glyph_id: u16,
    glyph_count: u16
}

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
struct Format2Header {
    format: u16,
    class_range_count: u16
}

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
struct ClassRangeRecord {
    start_glyph_id: u16,
    end_glyph_id: u16,
    class_id: u16
}

impl ClassDef {
    pub fn from_glyph_class(glyph_class: &GlyphClass, glyph_order: &GlyphOrder) -> Result<Self, GlyphOrderError> {
        let glyphs = glyph_class.iter_glyphs(glyph_order);

        glyphs.collect::<Result<_, GlyphOrderError>>()
            .map(Self)
    }

    fn encode_format_1(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let header = Format1Header {
            format: 1,
            start_glyph_id: self.0.iter().next().map(|x| *x).unwrap_or(0u16),
            glyph_count: self.0.len() as u16
        };

        buf.append(&header)?;

        for id in self.0.iter() {
            buf.append(id)?;
        }

        Ok(start)
    }

    fn encode_format_2(buf: &mut EncodeBuf, records: &[(u16, ops::Range<u16>)]) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let header = Format2Header {
            format: 2,
            class_range_count: records.len() as u16
        };

        buf.append(&header)?;

        for (class_id, range) in records {
            let range_record = ClassRangeRecord {
                start_glyph_id: range.start,
                end_glyph_id: range.end,
                class_id: *class_id
            };

            buf.append(&range_record)?;
        }

        Ok(start)
    }

    fn format_1_size(&self) -> usize {
        Format1Header::PACKED_LEN
            + (self.0.len() * u16::PACKED_LEN)
    }

    fn format_2_size(records: &[(u16, ops::Range<u16>)]) -> usize {
        Format2Header::PACKED_LEN
            + (records.len() * ClassRangeRecord::PACKED_LEN)
    }
}

impl TTFEncode for ClassDef {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let ranges: Vec<_> = self.0.iter()
            .map(|x| *x)
            .contiguous_ranges()
            .map(|(start, end)| (1u16, (start..end)))
            .collect();

        if self.format_1_size() < Self::format_2_size(&ranges) {
            self.encode_format_1(buf)
        } else {
            Self::encode_format_2(buf, &ranges)
        }
    }
}

// separate trait because we need the additional `use_zero_class` parameter
pub trait ClassDefTTFEncode {
    fn ttf_encode(&self, buf: &mut EncodeBuf, use_zero_class: bool) -> EncodeResult<usize>;
}

impl<'a, T> ClassDefTTFEncode for T
    where T: ops::Deref<Target = [&'a ClassDef]>
{
    fn ttf_encode(&self, buf: &mut EncodeBuf, use_zero_class: bool) -> EncodeResult<usize> {
        let classes =
            if use_zero_class && self.len() > 1 {
                &self[1..]
            } else {
                self
            };

        if let [single_class] = classes {
            return single_class.ttf_encode(buf);
        }

        let mut ranges = Vec::new();

        for (cls_id, cls) in classes.iter().enumerate() {
            cls.iter()
                .map(|x| *x)
                .contiguous_ranges()
                .map(|(start, end)| (cls_id as u16 + 1, (start..end)))
                .for_each(|r| ranges.push(r));
        }

        ClassDef::encode_format_2(buf, &ranges)
    }
}
