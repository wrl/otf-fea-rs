use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode_u16_be;
use crate::parse_model as pm;

#[derive(Debug)]
pub struct ScriptList(Vec<ScriptRecord>);

impl ScriptList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let count = decode_u16_be(bytes, 0);
        let mut records = Vec::new();

        for i in 0..count {
            let start = 2 + (i as usize * ScriptRecord::PACKED_LEN);
            let end = start + ScriptRecord::PACKED_LEN;

            let script_record = ScriptRecord::decode_from_be_bytes(
                &bytes[start..end]);

            records.push(script_record);
        }

        Self(records)
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct ScriptRecord {
    pub tag: pm::Tag,
    pub script_offset: u16
}
