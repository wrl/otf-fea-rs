use std::ops;
use std::collections::BTreeMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::util::*;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::util::*;


#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct CoverageHeader {
    format: u16,
    count: u16
}

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
struct GlyphRange {
    start: u16,
    end: u16,
    start_coverage_index: u16
}

fn decode_coverage<'a>(bytes: &'a [u8]) -> DecodeResult<impl Iterator<Item = u16> + 'a> {
    let header: CoverageHeader = decode_from_slice(bytes);
    let list_slice = &bytes[CoverageHeader::PACKED_LEN..];

    let glyphs_iter = match header.format {
        1 => Either2::A(decode_from_pool(header.count, list_slice)),

        2 => {
            let glyphs = decode_from_pool(header.count, list_slice)
                .flat_map(|r: GlyphRange|
                    r.start..(r.end + 1));

            Either2::B(glyphs)
        },

        _ => return Err(
            DecodeError::InvalidValue("format", "Coverage".into()))
    };

    Ok(glyphs_iter)
}

#[derive(Debug)]
pub struct CoverageLookup<T>(pub BTreeMap<u16, T>);

impl<T> Default for CoverageLookup<T> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<T> ops::Deref for CoverageLookup<T> {
    type Target = BTreeMap<u16, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for CoverageLookup<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl<T> CoverageLookup<T> {
    #[inline]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    #[inline]
    pub fn decode_with_lookup<I>(coverage_bytes: &[u8], lookup_iter: I) -> DecodeResult<Self>
        where I: Iterator<Item = T>
    {
        let decode_iter = decode_coverage(coverage_bytes)?;

        Ok(CoverageLookup(
            decode_iter.zip(lookup_iter)
                .collect()
        ))
    }
}

impl<Any> CoverageLookup<Any> {
    #[inline]
    fn format_1_size<'a, I>(iter: &'a I) -> usize
        where I: ExactSizeIterator<Item = u16>
    {
        iter.len() * u16::PACKED_LEN
    }

    #[inline]
    fn format_2_size<'a, I>(iter: &'a I) -> usize
        where I: ExactSizeIterator<Item = u16> + Clone
    {
        iter.clone()
            .contiguous_ranges()
            .count() * GlyphRange::PACKED_LEN
    }

    fn encode_format_1<'a>(iter: impl ExactSizeIterator<Item = u16>, buf: &mut EncodeBuf)
        -> EncodeResult<CoverageHeader>
    {
        let count = iter.len();

        for glyph_id in iter {
            buf.append(&glyph_id)?;
        }

        Ok(CoverageHeader {
            format: 1,
            count: count.checked_into("Coverage Type 1", "count")?
        })
    }

    fn encode_format_2<'a>(iter: impl ExactSizeIterator<Item = u16>, buf: &mut EncodeBuf)
        -> EncodeResult<CoverageHeader>
    {
        let mut start_coverage_index = 0u16;
        let mut count = 0u16;

        for range in iter.contiguous_ranges() {
            let glyph_range = GlyphRange {
                start: range.0,
                end: range.1,
                start_coverage_index
            };

            buf.append(&glyph_range)?;
            start_coverage_index += range.1 - range.0 + 1u16;
            count += 1;
        }

        Ok(CoverageHeader {
            format: 2,
            count
        })
    }

    pub fn encode<'a, I>(iter: I, buf: &mut EncodeBuf) -> EncodeResult<usize>
        where I: ExactSizeIterator<Item = u16> + Clone
    {
        let start = buf.bytes.len();

        buf.bytes.resize(start + CoverageHeader::PACKED_LEN, 0u8);

        if iter.len() < 4 || Self::format_1_size(&iter) < Self::format_2_size(&iter) {
            Self::encode_format_1(iter, buf)
        } else {
            Self::encode_format_2(iter, buf)
        }
            .and_then(|header| buf.encode_at(&header, start))
    }
}

impl<T> TTFEncode for CoverageLookup<T> {
    #[inline]
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        Self::encode(
            self.keys().map(|x| *x),
            buf)
    }
}
