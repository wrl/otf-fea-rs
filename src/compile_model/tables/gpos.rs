use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::TTFTable;

#[derive(Debug)]
pub struct GPOS {
    script_list: ScriptList,
    feature_list: FeatureList,
    lookup_list: LookupList<GPOSLookup>,
    feature_variations_offset: Option<u16>
}

#[derive(Debug)]
struct GPOSLookup {
    pub format: u16
}

impl TTFTable for GPOSLookup {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Result<Self, ()> {
        Ok(GPOSLookup {
            format: decode_u16_be(bytes, 0)
        })
    }
}

fn decode_from_be_bytes<T>(bytes: &[u8]) -> T
    where T: DecodeBE
{
    T::decode_from_be_bytes(&bytes[..T::PACKED_LEN])
}

impl GPOS {
    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let version: Version = decode_from_be_bytes(bytes);

        let offsets: Offsets = match (version.major, version.minor) {
            (1, 0) => Header_1_0::decode_from_be_bytes(bytes).into(),
            (1, 1) => Header_1_1::decode_from_be_bytes(bytes).into(),

            _ => return Err(())
        };

        Ok(GPOS {
            script_list: ScriptList::decode_from_be_bytes(&bytes[offsets.script as usize..]),
            feature_list: FeatureList::decode_from_be_bytes(&bytes[offsets.feature as usize..]),
            lookup_list: LookupList::decode_from_be_bytes(&bytes[offsets.lookup as usize..]),
            feature_variations_offset: offsets.feature_variations
        })
    }
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Version {
    major: u16,
    minor: u16
}

struct Offsets {
    script: u16,
    feature: u16,
    lookup: u16,
    feature_variations: Option<u16>
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_0 {
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
struct Header_1_1 {
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
