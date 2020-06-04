use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub struct Head {
    pub header: super::TTFTableHeader,

    pub major_version: u16,
    pub minor_version: u16,

    // FIXME: fixed-point
    pub font_revision: u32,

    pub checksum_adjustment: u32,
    pub magic_number: u32, // set to 0x5F0F3CF5
    pub flags: u16,

    pub units_per_em: u16,

    // FIXME: both LONGDATETIME
    pub created: u64,
    pub modified: u64,

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
