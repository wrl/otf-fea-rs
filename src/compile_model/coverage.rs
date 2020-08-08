use std::collections::BTreeMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::util::Either2;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;


impl<A, B, T> Iterator for Either2<A, B>
    where A: Iterator<Item = T>,
          B: Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self {
            Either2::A(inner) => inner.next(),
            Either2::B(inner) => inner.next()
        }
    }
}

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
pub struct GlyphRange {
    pub start: u16,
    pub end: u16,
    pub start_coverage_index: u16
}

fn decode_coverage<'a>(bytes: &'a [u8]) -> DecodeResult<impl Iterator<Item = u16> + 'a> {
    let format = decode_u16_be(bytes, 0);
    let count = decode_u16_be(bytes, 2);

    let list_slice = &bytes[4..];

    let glyphs_iter = match format {
        1 => Either2::A(decode_from_pool(count, list_slice)),

        2 => {
            let glyphs = decode_from_pool(count, list_slice)
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

trait Pred: Copy
{
    fn pred(self) -> Self;
}

impl Pred for u16 {
    fn pred(self) -> Self {
        self - 1
    }
}

struct ContiguousRanges<T, I>
    where T: Pred + Eq,
          I: Iterator<Item = T>
{
    inner: I,
    start: Option<T>
}

impl<T, I> Iterator for ContiguousRanges<T, I>
    where T: Pred + Eq + Copy,
          I: Iterator<Item = T>
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        let start = match self.start.take() {
            Some(x) => x,

            None => match self.inner.next() {
                Some(x) => x,
                None => return None
            }
        };

        let mut prev = start;

        for x in &mut self.inner {
            if prev != x.pred() {
                self.start = Some(x);
                return Some((start, prev));
            }

            prev = x;
        }

        Some((start, prev))
    }
}

fn contiguous_ranges<T, I>(inner: I) -> ContiguousRanges<T, I>
    where T: Pred + Eq,
          I: Iterator<Item = T>
{
    ContiguousRanges {
        inner,
        start: None
    }
}

impl<T> CoverageLookup<T> {
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.0.values()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
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

impl<T> TTFEncode for CoverageLookup<T> {
    #[inline]
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let format = 2u16;
        buf.append(&format)?;
        let count_offset = buf.bytes.len();
        buf.append(&0u16)?;

        // &u16 -> u16
        let glyphs = self.0.keys().map(|x| *x);
        let mut start_coverage_index = 0u16;
        let mut count = 0u16;

        for range in contiguous_ranges(glyphs) {
            let glyph_range = GlyphRange {
                start: range.0,
                end: range.1,
                start_coverage_index
            };

            buf.append(&glyph_range)?;
            start_coverage_index += range.1 - range.0 + 1u16;
            count += 1;
        }

        buf.encode_at(&count, count_offset)?;

        Ok(start)
    }
}
