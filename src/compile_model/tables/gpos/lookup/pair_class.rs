use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct PairPosFormat2Header {
    pub format: u16,
    pub coverage_offset: u16,
    pub value_format_1: u16,
    pub value_format_2: u16,
    pub class_def_1_offset: u16,
    pub class_def_2_offset: u16,
    pub class_1_count: u16,
    pub class_2_count: u16
}
