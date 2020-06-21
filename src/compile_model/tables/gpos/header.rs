use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub(super) struct Version {
    pub(super) major: u16,
    pub(super) minor: u16
}

pub(super) struct Offsets {
    pub(super) script: usize,
    pub(super) feature: usize,
    pub(super) lookup: usize,
    pub(super) feature_variations: Option<usize>
}

#[derive(PackedSize, EncodeBE, DecodeBE)]
pub(super) struct Header_1_0 {
    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16
}

impl From<Header_1_0> for Offsets {
    fn from(header: Header_1_0) -> Self {
        Self {
            script: header.script_list_offset as usize,
            feature: header.feature_list_offset as usize,
            lookup: header.lookup_list_offset as usize,
            feature_variations: None
        }
    }
}

#[derive(PackedSize, EncodeBE, DecodeBE)]
pub(super) struct Header_1_1 {
    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16,
    feature_variations_offset: u16
}

impl From<Header_1_1> for Offsets {
    fn from(header: Header_1_1) -> Self {
        Self {
            script: header.script_list_offset as usize,
            feature: header.feature_list_offset as usize,
            lookup: header.lookup_list_offset as usize,
            feature_variations: Some(header.feature_variations_offset as usize)
        }
    }
}
