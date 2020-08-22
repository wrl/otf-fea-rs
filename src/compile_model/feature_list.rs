use std::collections::BTreeMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;

use crate::Tag;

type LookupIndices = Vec<u16>;

#[derive(Debug)]
pub struct FeatureList(pub BTreeMap<Tag, LookupIndices>);

impl FeatureList {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn indices_for_tag(&self, tag: &Tag) -> &[u16] {
        match self.0.get(tag) {
            Some(i) => i,
            None => &[]
        }
    }

    pub fn indices_for_tag_mut(&mut self, tag: &Tag) -> &mut LookupIndices {
        self.0.entry(*tag)
            .or_default()
    }
}

impl TTFDecode for FeatureList {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let records_count = decode_u16_be(bytes, 0);
        let records = decode_from_pool(records_count, &bytes[2..]);

        let features = records.map(|r: FeatureRecord| {
            let table = &bytes[r.feature_offset as usize..];
            let lookup_index_count = decode_u16_be(table, 2);

            (r.tag,
             decode_from_pool(lookup_index_count, &table[4..])
                .collect())
        });

        Ok(Self(features.collect()))
    }
}

fn encode_feature_table(buf: &mut EncodeBuf, lookup_indices: &[u16]) -> EncodeResult<usize>
{
    let start = buf.bytes.len();

    let header = FeatureTable {
        params: 0,
        lookup_index_count: lookup_indices.len() as u16
    };

    buf.append(&header)?;

    for lookup_index in lookup_indices.iter() {
        buf.append(lookup_index)?;
    }

    Ok(start)
}

impl TTFEncode for FeatureList {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let len = self.0.len();

        buf.append(&(len as u16))?;

        let mut record_start = buf.bytes.len();

        buf.bytes.resize(record_start + (len * FeatureRecord::PACKED_LEN), 0u8);

        for (tag, lookup_indices) in self.0.iter() {
            if lookup_indices.len() == 0 {
                continue
            }

            let record = FeatureRecord {
                tag: *tag,
                feature_offset:
                    (encode_feature_table(buf, lookup_indices)? - start) as u16
            };

            buf.encode_at(&record, record_start)?;
            record_start += FeatureRecord::PACKED_LEN;
        }

        Ok(start)
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct FeatureRecord {
    pub tag: Tag,
    pub feature_offset: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct FeatureTable {
    pub params: u16,
    pub lookup_index_count: u16
}
