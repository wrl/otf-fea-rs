use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::parse_model::*;
use crate::compile_model::util::*;

#[derive(Debug, Clone, PackedSize, EncodeBE, DecodeBE)]
pub struct Head {
    pub major_version: u16,
    pub minor_version: u16,

    // FIXME: fixed-point
    pub font_revision: Fixed1616,

    pub checksum_adjustment: u32,
    pub magic_number: u32, // set to 0x5F0F3CF5
    pub flags: u16,

    pub units_per_em: u16,

    pub created: LongDateTime,
    pub modified: LongDateTime,

    pub x_min: u16,
    pub y_min: u16,
    pub x_max: u16,
    pub y_max: u16,

    pub mac_style: u16,

    pub lowest_rec_ppem: u16,

    // deprecated (set to 2)
    pub font_direction_hint: i16,

    pub index_to_loc_format: i16,

    // set to 0 for "current format"
    pub glyph_data_format: i16,
}

impl Head {
    pub fn new() -> Self {
        Self {
            major_version: 1,
            minor_version: 0,

            font_revision: Fixed1616::from_bits(0),

            checksum_adjustment: 0,
            magic_number: 0x5F0F3CF5,
            flags: 0,

            created: LongDateTime::new(),
            modified: LongDateTime::new(),

            units_per_em: 0,

            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,

            mac_style: 0,

            lowest_rec_ppem: 0,

            font_direction_hint: 2,

            index_to_loc_format: 0,
            glyph_data_format: 0
        }
    }

    pub fn from_parsed_table(statements: &[TableStatement]) -> Self {
        let revision = statements.iter()
            .map(|s| {
                use TableStatement::*;

                match s {
                    FontRevision(head::FontRevision(f)) => *f,
                    _ => unreachable!()
                }
            })
            .last()
            .unwrap_or(0f64);

        let mut res = Self::new();
        res.font_revision = Fixed1616::from_f32(revision as f32);
        res
    }
}
