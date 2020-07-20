use endian_codec::{PackedSize, DecodeBE};

use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::{
    TTFTable,
    TTFEncode,
    CompileResult,
    EncodeBuf
};

mod header;
use header::*;

mod lookup;
use lookup::*;

#[derive(Debug)]
pub struct GPOS {
    pub script_list: ScriptList,
    pub feature_list: FeatureList,
    pub lookup_list: LookupList<GPOSSubtable>,
    pub feature_variations: Option<usize>
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
    fn encode_as_be_bytes(&self, _buf: &mut Vec<u8>) -> Result<(), ()> {
        Ok(())
    }
}

impl TTFEncode for GPOS {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> CompileResult<usize> {
        let header_size =
            self.feature_variations
                .map(|_| Header_1_1::PACKED_LEN)
                .unwrap_or(Header_1_0::PACKED_LEN);

        let start = buf.bytes.len();
        buf.bytes.resize(header_size, 0u8);

        let offsets = Offsets {
            script: buf.append(&self.script_list)?,
            feature: buf.append(&self.feature_list)?,
            lookup: buf.append(&self.lookup_list)?,
            feature_variations: None
        };

        let header: Header_1_0 = offsets.into();
        buf.encode_at(&header, start)?;

        Ok(start)
    }
}
