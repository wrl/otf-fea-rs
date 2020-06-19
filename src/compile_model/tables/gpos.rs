use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::script_list::*;

#[derive(Debug)]
pub struct GPOS {
    script_list: ScriptList,
    feature_list_offset: u16,
    lookup_list_offset: u16,
    feature_variations_offset: Option<u16>
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

        let gpos = match (version.major, version.minor) {
            (1, 0) => {
                let header: Header_1_0 = decode_from_be_bytes(bytes);

                GPOS {
                    script_list: ScriptList::decode_from_be_bytes(
                                     &bytes[header.script_list_offset as usize..]),
                    feature_list_offset: header.feature_list_offset,
                    lookup_list_offset: header.lookup_list_offset,
                    feature_variations_offset: None,
                }
            },

            (1, 1) => {
                let header: Header_1_1 = decode_from_be_bytes(bytes);

                GPOS {
                    script_list: ScriptList::decode_from_be_bytes(
                                     &bytes[header.script_list_offset as usize..]),
                    feature_list_offset: header.feature_list_offset,
                    lookup_list_offset: header.lookup_list_offset,
                    feature_variations_offset: Some(header.feature_variations_offset),
                }
            }

            _ => return Err(())
        };

        Ok(gpos)
    }
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Version {
    major: u16,
    minor: u16
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_0 {
    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16
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
