#![allow(non_camel_case_types)]

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

pub mod gpos;
pub mod gdef;
pub mod head;

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct SFNTHeader {
    // 0x00010000 for ttf, "OTTO" for otf
    pub version: u32,

    pub num_tables: u16,

    // (maximum power of 2 <= num_tables) * 16
    pub search_range: u16,

    // log2(maximum power of 2 <= num_tables)
    pub entry_selector: u16,

    // (num_tables * 16) - search_range
    pub range_shift: u16
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct TTFTableHeader {
    tag: u32,
    checksum: u32,
    offset_from_start_of_file: u32,
    length: u32
}
