use std::cmp::Ordering;
use std::convert::TryFrom;
use std::collections::BTreeMap;
use std::collections::btree_map::Values;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;


#[derive(Eq, PartialOrd, Debug, PackedSize, DecodeBE, EncodeBE)]
pub struct GlyphRange {
    pub start: u16,
    pub end: u16,
    pub start_coverage_index: u16
}

impl Ord for GlyphRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialEq for GlyphRange {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

#[derive(Debug)]
pub enum CoverageLookup<T> {
    Glyphs(BTreeMap<u16, T>),
    GlyphRanges(BTreeMap<GlyphRange, T>)
}

#[derive(Debug)]
pub enum CoverageValues<'a, T> {
    OfGlyphs(Values<'a, u16, T>),
    OfGlyphRanges(Values<'a, GlyphRange, T>)
}

impl<'a, T> Iterator for CoverageValues<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self {
            CoverageValues::OfGlyphs(g) => g.next(),
            CoverageValues::OfGlyphRanges(r) => r.next()
        }
    }
}

impl<T> CoverageLookup<T> {
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        match self {
            CoverageLookup::Glyphs(g) =>
                CoverageValues::OfGlyphs(g.values()),
            CoverageLookup::GlyphRanges(r) =>
                CoverageValues::OfGlyphRanges(r.values())
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            CoverageLookup::Glyphs(g) => g.len(),
            CoverageLookup::GlyphRanges(r) => r.len()
        }
    }
}

#[derive(Debug)]
pub enum Coverage {
    Glyphs(Vec<u16>),
    GlyphRanges(Vec<GlyphRange>)
}

impl TTFDecode for Coverage {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let format = decode_u16_be(bytes, 0);
        let count = decode_u16_be(bytes, 2);

        let list_slice = &bytes[4..];

        Ok(match format {
            1 => Self::Glyphs(
                decode_from_pool(count, list_slice).collect()),

            2 => Self::GlyphRanges(
                decode_from_pool(count, list_slice).collect()),

            _ => return Err(
                DecodeError::InvalidValue("format", "Coverage".into()))
        })
    }
}

#[inline]
fn encode_coverage<T: EncodeBE>(buf: &mut EncodeBuf, format: u16, data: &[T])
        -> EncodeResult<usize> {
    let start = buf.bytes.len();

    buf.append(&format)?;

    // FIXME: generalised u16 writing?
    let count = u16::try_from(data.len())
        .map_err(|_| EncodeError::U16Overflow {
            scope: "Coverage".into(),
            item: "count",
            value: data.len()
        })?;

    buf.append(&count)?;

    for val in data {
        buf.append(val)?;
    }

    Ok(start)
}

impl TTFEncode for Coverage {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            Self::Glyphs(ref glyphs) => encode_coverage(buf, 1, glyphs),
            Self::GlyphRanges(ref ranges) => encode_coverage(buf, 2, ranges)
        }
    }
}

