use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::Tag;
use crate::compile_model::TTFVersion;


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

impl TTFOffsetTable {
    pub fn new(version: TTFVersion, num_tables: u16) -> Self {
        let max_power_of_two = 15u16.saturating_sub(
            num_tables.leading_zeros() as u16);

        let search_range = (1 << max_power_of_two) * 16;
        let entry_selector = max_power_of_two;
        let range_shift = (num_tables * 16).saturating_sub(search_range);

        Self {
            version,
            num_tables,
            search_range,
            entry_selector,
            range_shift
        }
    }
}


#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub struct TTFTableRecord {
    pub tag: Tag,
    pub checksum: u32,
    pub offset_from_start_of_file: u32,
    pub length: u32
}

impl TTFTableRecord {
    pub fn table_data<'a>(&self, whole_file: &'a [u8]) -> &'a [u8] {
        let (start, end) = (
            self.offset_from_start_of_file as usize,
            (self.offset_from_start_of_file + self.length) as usize
        );

        &whole_file[start..end]
    }
}
