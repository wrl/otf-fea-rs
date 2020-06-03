use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_0 {
    header: super::TTFTableHeader,

    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_1 {
    header: super::TTFTableHeader,

    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16,
    feature_variations_offset: u16
}
