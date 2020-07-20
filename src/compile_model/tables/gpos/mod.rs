use endian_codec::{PackedSize, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;

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

impl TTFDecode for GPOS {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> Self {
        let version: Version = decode_from_slice(bytes);

        let offsets: Offsets = match (version.major, version.minor) {
            (1, 0) => Header_1_0::decode_from_be_bytes(bytes).into(),
            (1, 1) => Header_1_1::decode_from_be_bytes(bytes).into(),

            // FIXME: extend TTFDecode to return a result
            _ => panic!()
        };

        GPOS {
            script_list: ScriptList::ttf_decode(&bytes[offsets.script..]),
            feature_list: FeatureList::ttf_decode(&bytes[offsets.feature..]),
            lookup_list: LookupList::ttf_decode(&bytes[offsets.lookup..]),
            feature_variations: offsets.feature_variations
        }
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
