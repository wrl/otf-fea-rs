use endian_codec::{PackedSize, EncodeBE, DecodeBE};
use fixed::{types::I16F16};

#[derive(Debug, Copy, Clone)]
pub struct Fixed1616(I16F16);

impl Fixed1616 {
    pub const fn from_bits(bits: i32) -> Self {
        Self(I16F16::from_bits(bits))
    }

    pub const fn to_bits(self) -> i32 {
        self.0.to_bits()
    }

    pub fn from_f32(src: f32) -> Self {
        Self(I16F16::from_num(src))
    }

    pub fn to_f32(self) -> f32 {
        self.0.to_num()
    }
}

impl PackedSize for Fixed1616 {
    const PACKED_LEN: usize = 4;
}

impl EncodeBE for Fixed1616 {
    #[inline]
    fn encode_as_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&(self.0.to_be_bytes()));
    }
}

impl DecodeBE for Fixed1616 {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Self(I16F16::from_be_bytes(arr))
    }
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub struct Head {
    pub major_version: u16,
    pub minor_version: u16,

    // FIXME: fixed-point
    pub font_revision: Fixed1616,

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

impl Head {
    pub fn new() -> Self {
        Self {
            major_version: 1,
            minor_version: 0,

            font_revision: Fixed1616::from_bits(0),

            checksum_adjustment: 0,
            magic_number: 0x5F0F3CF5,
            flags: 0,

            created: 0,
            modified: 0,

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
}
