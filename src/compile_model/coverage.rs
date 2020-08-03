use std::convert::TryFrom;
use std::collections::BTreeMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::util::Either2;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;


#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
pub struct GlyphRange {
    pub start: u16,
    pub end: u16,
    pub start_coverage_index: u16
}

#[derive(Debug)]
pub struct CoverageLookup<T>(pub BTreeMap<u16, T>);

impl<T> CoverageLookup<T> {
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.0.values()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Coverage(pub Vec<u16>);

impl Coverage {
    pub(crate) fn into_glyphs(self) -> Vec<u16> {
        self.0
    }
}

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

impl TTFDecode for Coverage {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let format = decode_u16_be(bytes, 0);
        let count = decode_u16_be(bytes, 2);

        let list_slice = &bytes[4..];

        let glyphs = match format {
            1 => Either2::A(decode_from_pool(count, list_slice)),

            2 => {
                let glyphs = decode_from_pool(count, list_slice)
                    .flat_map(|r: GlyphRange| {
                        r.start..(r.end + 1)
                    });

                Either2::B(glyphs)
            },

            _ => return Err(
                DecodeError::InvalidValue("format", "Coverage".into()))
        };

        Ok(Coverage(glyphs.collect()))
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
        encode_coverage(buf, 1, &self.0)
    }
}
