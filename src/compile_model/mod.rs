#![allow(non_camel_case_types)]

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

pub mod gpos;
pub mod gdef;
pub mod head;

pub mod util;

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct TTFOffsetTable {
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
pub struct TTFTableHeader {
    pub tag: u32,
    pub checksum: u32,
    pub offset_from_start_of_file: u32,
    pub length: u32
}
