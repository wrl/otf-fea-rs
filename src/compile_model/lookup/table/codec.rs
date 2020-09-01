use std::collections::HashMap;

use endian_codec::{PackedSize, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;

use super::*;
use super::header::*;

impl<L: TTFDecode> TTFDecode for LookupTable<L> {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let version: Version = decode_from_slice(bytes);

        let offsets: Offsets = match (version.major, version.minor) {
            (1, 0) => Header_1_0::decode_from_be_bytes(bytes).into(),
            (1, 1) => Header_1_1::decode_from_be_bytes(bytes).into(),

            _ => return Err(DecodeError::InvalidValue("version", "LookupTable header".into()))
        };

        let script_bytes = &bytes[offsets.script..];
        let feature_bytes = &bytes[offsets.feature..];
        let lookup_bytes = &bytes[offsets.lookup..];

        Ok(LookupTable {
            script_list: ScriptList::ttf_decode(script_bytes, feature_bytes)?,
            feature_list: FeatureList::ttf_decode(feature_bytes)?,
            lookup_list: LookupList::ttf_decode(lookup_bytes)?,
            feature_variations: offsets.feature_variations,

            named_lookups: HashMap::new()
        })
    }
}

impl<L: TTFEncode> TTFEncode for LookupTable<L> {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let header_size =
            self.feature_variations
                .map(|_| Header_1_1::PACKED_LEN)
                .unwrap_or(Header_1_0::PACKED_LEN);

        let start = buf.bytes.len();
        buf.bytes.resize(header_size, 0u8);

        let offsets = Offsets {
            script: self.script_list.ttf_encode(buf, &self.feature_list)?,
            feature: self.feature_list.ttf_encode(buf)?,
            lookup: self.lookup_list.ttf_encode(buf)?,
            feature_variations: None
        };

        let header: Header_1_0 = offsets.into();
        buf.encode_at(&header, start)?;

        Ok(start)
    }
}
