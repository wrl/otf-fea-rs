use std::ops;
use std::collections::BTreeMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::util::*;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;


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

#[derive(Debug, Default)]
pub struct CoverageLookup<T>(pub BTreeMap<u16, T>);

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

    fn format_1_size(&self) -> usize {
        self.len() * u16::PACKED_LEN
    }

    fn format_2_size(&self) -> usize {
        self.keys()
            .map(|x| *x)
            .contiguous_ranges()
            .count() * GlyphRange::PACKED_LEN
    }

    fn encode_format_1(&self, buf: &mut EncodeBuf) -> EncodeResult<CoverageHeader> {
        for glyph_id in self.keys() {
            buf.append(glyph_id)?;
        }

        Ok(CoverageHeader {
            format: 1,
            count: self.len() as u16
        })
    }

    fn encode_format_2(&self, buf: &mut EncodeBuf) -> EncodeResult<CoverageHeader> {
        let mut start_coverage_index = 0u16;
        let mut count = 0u16;

        for range in self.keys().map(|x| *x).contiguous_ranges() {
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
}

impl<T> TTFEncode for CoverageLookup<T> {
    #[inline]
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.bytes.resize(start + CoverageHeader::PACKED_LEN, 0u8);

        if self.len() < 4 || self.format_1_size() < self.format_2_size() {
            self.encode_format_1(buf)
        } else {
            self.encode_format_2(buf)
        }
            .and_then(|header| buf.encode_at(&header, start))
    }
}
