use endian_codec::DecodeBE;

use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::TTFTable;

mod header;
use header::*;

mod lookup;
use lookup::*;

#[derive(Debug)]
pub struct GPOS {
    script_list: ScriptList,
    feature_list: FeatureList,
    lookup_list: LookupList<GPOSLookup>,
    feature_variations_offset: Option<usize>
}

impl TTFTable for GPOS {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let version: Version = decode_from_slice(bytes);

        let offsets: Offsets = match (version.major, version.minor) {
            (1, 0) => Header_1_0::decode_from_be_bytes(bytes).into(),
            (1, 1) => Header_1_1::decode_from_be_bytes(bytes).into(),

            _ => return Err(())
        };

        Ok(GPOS {
            script_list: ScriptList::decode_from_be_bytes(&bytes[offsets.script..]),
            feature_list: FeatureList::decode_from_be_bytes(&bytes[offsets.feature..]),
            lookup_list: LookupList::decode_from_be_bytes(&bytes[offsets.lookup..]),
            feature_variations_offset: offsets.feature_variations
        })
    }
}
