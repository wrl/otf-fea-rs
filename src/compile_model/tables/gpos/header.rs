use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub(crate) struct Version {
    pub(crate) major: u16,
    pub(crate) minor: u16
}

pub(crate) struct Offsets {
    pub(crate) script: u16,
    pub(crate) feature: u16,
    pub(crate) lookup: u16,
    pub(crate) feature_variations: Option<u16>
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub(crate) struct Header_1_0 {
    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16
}

impl From<Header_1_0> for Offsets {
    fn from(header: Header_1_0) -> Self {
        Self {
            script: header.script_list_offset,
            feature: header.feature_list_offset,
            lookup: header.lookup_list_offset,
            feature_variations: None
        }
    }
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub(crate) struct Header_1_1 {
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
            script: header.script_list_offset,
            feature: header.feature_list_offset,
            lookup: header.lookup_list_offset,
            feature_variations: Some(header.feature_variations_offset)
        }
    }
}
