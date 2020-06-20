use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::parse_model as pm;

#[derive(Debug)]
pub struct FeatureList(Vec<Feature>);

#[derive(Debug)]
pub struct Feature {
    pub tag: pm::Tag,
    pub lookup_indices: Vec<u16>
}

impl FeatureList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let records_count = decode_u16_be(bytes, 0);
        let records = decode_from_pool(records_count, &bytes[2..]);

        let features = records.map(|r: FeatureRecord| {
            let feature_table = &bytes[r.feature_offset as usize..];
            let lookup_index_count = decode_u16_be(feature_table, 2);

            Feature {
                tag: r.tag,
                lookup_indices: decode_from_pool(lookup_index_count, &feature_table[4..])
                    .collect()
            }
        });

        Self(features.collect())
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct FeatureRecord {
    pub tag: pm::Tag,
    pub feature_offset: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct FeatureTable {
    pub params: u16,
    pub lookup_index_count: u16
}
