use std::convert::TryFrom;

use bitflags::bitflags;
use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;

////
// LookupList
////

#[derive(Debug)]
pub struct LookupList<T>(Vec<Lookup<T>>);

impl<T> LookupList<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl<T: TTFDecode> TTFDecode for LookupList<T> {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let records_count = decode_u16_be(bytes, 0);
        let records = decode_from_pool(records_count, &bytes[2..]);

        let lookups = records
            .map(|offset: u16|
                Lookup::ttf_decode(&bytes[offset as usize..]));

        lookups.collect::<DecodeResult<_>>()
            .map(Self)
    }
}

impl<T: TTFEncode> TTFEncode for LookupList<T> {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.append(&(self.0.len() as u16))?;
        let mut record_offset = buf.bytes.len();
        buf.bytes.resize(record_offset +
            (self.0.len() * u16::PACKED_LEN), 0u8);

        for lookup in &self.0 {
            let offset = (buf.append(lookup)? - start) as u16;

            buf.encode_at(&offset, record_offset)?;
            record_offset += u16::PACKED_LEN;
        }

        Ok(start)
    }
}

////
// Lookup
////

bitflags! {
    pub struct LookupFlags: u16 {
        const RIGHT_TO_LEFT = 0x0001;
        const IGNORE_BASE_GLYPHS = 0x0002;
        const IGNORE_LIGATURES = 0x0004;
        const IGNORE_MARKS = 0x0008;
        const USE_MARK_FILTERING_SET = 0x0010;

        const MARK_ATTACHMENT_TYPE = 0xFF00;
    }
}

#[derive(Debug)]
pub struct Lookup<T> {
    pub lookup_type: u16,
    pub lookup_flags: LookupFlags,
    pub mark_filtering_set: Option<u16>,

    pub subtables: Vec<T>,
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct LookupTableHeader {
    pub lookup_type: u16,
    pub lookup_flags: u16,
    pub subtable_count: u16
}

impl<T: TTFDecode> TTFDecode for Lookup<T> {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let header = decode_from_slice::<LookupTableHeader>(bytes);

        let lookup_flags = LookupFlags::from_bits_truncate(header.lookup_flags);

        let subtables =
            decode_from_pool(header.subtable_count, &bytes[LookupTableHeader::PACKED_LEN..])
            .map(|offset: u16|
                T::ttf_decode(&bytes[offset as usize..]))
            .collect::<DecodeResult<_>>()?;

        let mark_filtering_set =
            if lookup_flags.contains(LookupFlags::USE_MARK_FILTERING_SET) {
                Some(decode_u16_be(bytes,
                    LookupTableHeader::PACKED_LEN
                    + (header.subtable_count as usize * 2)))
            } else {
                None
            };

        Ok(Lookup {
            lookup_type: header.lookup_type,
            lookup_flags,
            mark_filtering_set,

            subtables
        })
    }
}

impl<T: TTFEncode> TTFEncode for Lookup<T> {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let mut flags = self.lookup_flags;

        flags.set(LookupFlags::USE_MARK_FILTERING_SET,
            self.mark_filtering_set.is_some());

        let header = LookupTableHeader {
            lookup_type: self.lookup_type,
            lookup_flags: self.lookup_flags.bits(),
            subtable_count: self.subtables.len() as u16
        };

        buf.append(&header)?;

        let mut offset = buf.bytes.len();
        buf.bytes.resize(start + (u16::PACKED_LEN * self.subtables.len()), 0u8);

        if let Some(mfs) = self.mark_filtering_set {
            buf.append(&mfs)?;
        }

        for subtable in &self.subtables {
            let st_offset = buf.append(subtable)? as u16;
            buf.encode_at(&st_offset, offset)?;
            offset += u16::PACKED_LEN;
        }

        Ok(start)
    }
}

////
// Coverage
////

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
pub struct GlyphRange {
    pub start: u16,
    pub end: u16,
    pub start_coverage_index: u16
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
