use endian_codec::{PackedSize, DecodeBE};

use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::TTFTable;

mod header;
use header::*;

mod lookup;
use lookup::*;

#[derive(Debug)]
pub struct GPOS {
    script_list: ScriptList,
    feature_list: FeatureList,
    lookup_list: LookupList<GPOSSubtable>,
    feature_variations: Option<usize>
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
            feature_variations: offsets.feature_variations
        })
    }

    #[inline]
    fn encode_as_be_bytes(&self, buf: &mut Vec<u8>) -> Result<(), ()> {
        let header_size =
            self.feature_variations
                .map(|_| Header_1_1::PACKED_LEN)
                .unwrap_or(Header_1_0::PACKED_LEN);

        buf.resize(header_size, 0u8);

        let mut offsets = Offsets {
            script: 0,
            feature: 0,
            lookup: 0,
            feature_variations: None
        };

        offsets.script = buf.len();
        self.script_list.encode_as_be_bytes(buf);
        offsets.feature = buf.len();
        self.feature_list.encode_as_be_bytes(buf);

        let header: Header_1_0 = offsets.into();
        encode_to_slice(buf, &header);

        Ok(())
    }
}