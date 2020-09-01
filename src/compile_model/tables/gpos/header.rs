use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub(crate) struct Version {
    pub(crate) major: u16,
    pub(crate) minor: u16
}

pub(crate) struct Offsets {
    pub(crate) script: usize,
    pub(crate) feature: usize,
    pub(crate) lookup: usize,
    pub(crate) feature_variations: Option<usize>
}

impl From<Offsets> for Header_1_0 {
    fn from(offsets: Offsets) -> Self {
        Self {
            major: 1,
            minor: 0,
            script_list_offset: offsets.script as u16,
            feature_list_offset: offsets.feature as u16,
            lookup_list_offset: offsets.lookup as u16,
        }
    }
}

impl From<Offsets> for Header_1_1 {
    fn from(offsets: Offsets) -> Self {
        Self {
            major: 1,
            minor: 1,
            script_list_offset: offsets.script as u16,
            feature_list_offset: offsets.feature as u16,
            lookup_list_offset: offsets.lookup as u16,
            feature_variations_offset: offsets.feature_variations.unwrap_or(0) as u16,
        }
    }
}

#[derive(PackedSize, EncodeBE, DecodeBE)]
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
            script: header.script_list_offset as usize,
            feature: header.feature_list_offset as usize,
            lookup: header.lookup_list_offset as usize,
            feature_variations: None
        }
    }
}

#[derive(PackedSize, EncodeBE, DecodeBE)]
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
            script: header.script_list_offset as usize,
            feature: header.feature_list_offset as usize,
            lookup: header.lookup_list_offset as usize,
            feature_variations: Some(header.feature_variations_offset as usize)
        }
    }
}
