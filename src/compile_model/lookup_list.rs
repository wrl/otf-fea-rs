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
    fn ttf_decode(bytes: &[u8]) -> Self {
        let records_count = decode_u16_be(bytes, 0);
        let records = decode_from_pool(records_count, &bytes[2..]);

        let lookups = records
            .map(|offset: u16|
                Lookup::ttf_decode(&bytes[offset as usize..]));

        Self(lookups.collect())
    }
}

impl<T: TTFEncode> TTFEncode for LookupList<T> {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> CompileResult<usize> {
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
    fn ttf_decode(bytes: &[u8]) -> Self {
        let header = decode_from_slice::<LookupTableHeader>(bytes);

        let lookup_flags = LookupFlags::from_bits_truncate(header.lookup_flags);
        let mark_filtering_set =
            if lookup_flags.contains(LookupFlags::USE_MARK_FILTERING_SET) {
                Some(decode_u16_be(bytes,
                    LookupTableHeader::PACKED_LEN
                    + (header.subtable_count as usize * 2)))
            } else {
                None
            };

        let subtables =
            decode_from_pool(header.subtable_count, &bytes[LookupTableHeader::PACKED_LEN..])
            .map(|offset: u16| {
                T::ttf_decode(&bytes[offset as usize..])
            })
            .collect();

        Lookup {
            lookup_type: header.lookup_type,
            lookup_flags,
            mark_filtering_set,

            subtables
        }
    }
}

impl<T: TTFEncode> TTFEncode for Lookup<T> {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> CompileResult<usize> {
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

        if let Some(mfs) = self.mark_filtering_set {
            buf.append(&mfs)?;
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

#[allow(dead_code)]
impl Coverage {
    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Option<Self> {
        let format = decode_u16_be(bytes, 0);
        let count = decode_u16_be(bytes, 2);

        let list_slice = &bytes[4..];

        Some(match format {
            1 => Self::Glyphs(
                decode_from_pool(count, list_slice).collect()),

            2 => Self::GlyphRanges(
                decode_from_pool(count, list_slice).collect()),

            // FIXME: pass errors up
            _ => return None
        })
    }
}
