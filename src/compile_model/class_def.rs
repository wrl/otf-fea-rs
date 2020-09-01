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

    #[inline]
    fn first_and_last_glyphs(&self) -> (u16, u16) {
        // https://github.com/rust-lang/rust/issues/62924
        // why is this nightly-only, what the hell

        let mut iter = self.0.iter();

        iter.clone().next()
            .zip(iter.next_back())
            .map(|(a, b)| (*a, *b))
            .unwrap_or((0, 0))
    }

    // FIXME: only encodes class id 1 right now.
    //        should be easy enough to extend to multiple classes
    fn encode_format_1(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let (first, last) = self.first_and_last_glyphs();

        let header = Format1Header {
            format: 1,
            start_glyph_id: first,
            glyph_count: last - first + 1
        };

        buf.append(&header)?;

        for id in first..(last + 1) {
            if self.0.contains(&id) {
                buf.append(&1u16)?;
            } else {
                buf.append(&0u16)?;
            }
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
        let (first, last) = self.first_and_last_glyphs();

        Format1Header::PACKED_LEN
            + ((last - first + 1) as usize * u16::PACKED_LEN)
    }

    fn format_2_size(range_count: usize) -> usize {
        Format2Header::PACKED_LEN
            + (range_count * ClassRangeRecord::PACKED_LEN)
    }

    pub fn smallest_encoded_size(&self) -> usize {
        let ranges_count = self.0.iter()
            .map(|x| *x)
            .contiguous_ranges()
            .count();

        self.format_1_size()
            .min(Self::format_2_size(ranges_count))
    }
}

impl TTFEncode for ClassDef {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let ranges: Vec<_> = self.0.iter()
            .map(|x| *x)
            .contiguous_ranges()
            .map(|(start, end)| (1u16, (start..end)))
            .collect();

        if self.format_1_size() < Self::format_2_size(ranges.len()) {
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
            if use_zero_class && self.len() > 0 {
                &self[1..]
            } else {
                self
            };

        if self.len() == 0 {
            return ClassDef::encode_format_2(buf, &[]);
        }

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
