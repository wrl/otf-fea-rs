use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::{
    TTFEncode,
    EncodeBuf
};

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
            let table = &bytes[r.feature_offset as usize..];
            let lookup_index_count = decode_u16_be(table, 2);

            Feature {
                tag: r.tag,
                lookup_indices:
                    decode_from_pool(lookup_index_count, &table[4..])
                        .collect()
            }
        });

        Self(features.collect())
    }
}

fn encode_feature_table(buf: &mut EncodeBuf, feature: &Feature) -> Result<usize, ()>
{
    let start = buf.bytes.len();

    let header = FeatureTable {
        params: 0,
        lookup_index_count: feature.lookup_indices.len() as u16
    };

    buf.append(&header)?;

    for lookup_index in feature.lookup_indices.iter() {
        buf.append(lookup_index)?;
    }

    Ok(start)
}

impl TTFEncode for FeatureList {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> Result<usize, ()> {
        let start = buf.bytes.len();
        let len = self.0.len();

        buf.append(&(len as u16))?;

        let mut record_start = buf.bytes.len();

        buf.bytes.resize(start + (len * FeatureRecord::PACKED_LEN), 0u8);

        for feature in self.0.iter() {
            let record = FeatureRecord {
                tag: feature.tag,
                feature_offset: encode_feature_table(buf, feature)? as u16,
            };

            buf.encode_at(&record, record_start)?;
            record_start += FeatureRecord::PACKED_LEN;
        }

        Ok(start)
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
