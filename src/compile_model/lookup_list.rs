use bitflags::bitflags;
use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;

////
// LookupList
////

#[derive(Debug)]
pub struct LookupList(Vec<Lookup>);

impl LookupList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let records_count = decode_u16_be(bytes, 0);
        let records = decode_from_pool(records_count, &bytes[2..]);

        let lookups = records
            .map(|offset: u16|
                Lookup::decode_from_be_bytes(&bytes[offset as usize..]));

        Self(lookups.collect())
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
pub struct Lookup {
    pub lookup_type: u16,
    pub lookup_flags: LookupFlags,

    pub subtable_offsets: Vec<u16>
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct LookupTableHeader {
    pub lookup_type: u16,
    pub lookup_flags: u16,
    pub subtable_count: u16
}

impl Lookup {
    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let header = decode_from_slice::<LookupTableHeader>(bytes);
        let subtable_offsets =
            decode_from_pool(header.subtable_count, &bytes[LookupTableHeader::PACKED_LEN..])
            .collect();

        Lookup {
            lookup_type: header.lookup_type,
            lookup_flags: LookupFlags::from_bits_truncate(header.lookup_flags),
            subtable_offsets
        }
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
