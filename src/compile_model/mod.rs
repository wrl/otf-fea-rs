#![allow(non_camel_case_types)]

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::parse_model::Tag;

pub mod gpos;
pub mod gdef;
pub mod head;

pub mod util;

#[derive(Debug, Copy, Clone)]
pub enum TTFVersion {
    TTF,
    OTF,
    Unknown(u32)
}

impl PackedSize for TTFVersion {
    const PACKED_LEN: usize = 4;
}

impl EncodeBE for TTFVersion {
    #[inline]
    fn encode_as_be_bytes(&self, bytes: &mut [u8]) {
        let repr = match self {
            TTFVersion::TTF => 0x00010000u32,
            TTFVersion::OTF => 0x4f54544fu32,
            TTFVersion::Unknown(r) => *r // panic?
        };

        bytes.copy_from_slice(&(repr.to_be_bytes()));
    }
}

impl DecodeBE for TTFVersion {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);

        let repr = u32::from_be_bytes(arr);

        match repr {
            0x00010000u32 => TTFVersion::TTF,
            0x4f54544fu32 => TTFVersion::OTF,
            r => TTFVersion::Unknown(r)
        }
    }
}



#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct TTFOffsetTable {
    pub version: TTFVersion,

    pub num_tables: u16,

    // (maximum power of 2 <= num_tables) * 16
    pub search_range: u16,

    // log2(maximum power of 2 <= num_tables)
    pub entry_selector: u16,

    // (num_tables * 16) - search_range
    pub range_shift: u16
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub struct TTFTableRecord {
    pub tag: Tag,
    pub checksum: u32,
    pub offset_from_start_of_file: u32,
    pub length: u32
}
