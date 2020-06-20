use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::parse_model as pm;

#[derive(Debug)]
pub struct ScriptList(Vec<ScriptRecord>);

impl ScriptList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let records = decode_from_pool(decode_u16_be(bytes, 0), &bytes[2..]);
        Self(records.collect())
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct ScriptRecord {
    pub tag: pm::Tag,
    pub script_offset: u16
}
