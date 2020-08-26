use std::collections::{
    HashMap,
    BTreeSet
};

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::*;

use crate::compile_model::feature_list::{
    FeatureRecord,
    FeatureList
};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;

#[derive(Debug)]
pub struct ScriptList(HashMap<ScriptTag, Script>);

impl ScriptList {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[inline]
    pub fn script_for_tag(&self, tag: &ScriptTag) -> Option<&Script> {
        self.0.get(tag)
    }

    #[inline]
    pub fn script_for_tag_mut(&mut self, tag: &ScriptTag) -> &mut Script {
        self.0.entry(*tag)
            .or_insert_with(|| Script {
                default_lang_sys: LangSys {
                    required_feature: None,
                    features: BTreeSet::new()
                },

                lang_sys: HashMap::new()
            })
    }
}

#[derive(Debug)]
pub struct Script {
    pub default_lang_sys: LangSys,
    pub lang_sys: HashMap<Tag, LangSys>
}

#[derive(Debug)]
pub struct LangSys {
    pub required_feature: Option<FeatureTag>,
    pub features: BTreeSet<FeatureTag>
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct ScriptRecord {
    tag: ScriptTag,
    script_offset: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct ScriptTable {
    default_lang_sys: u16,
    lang_sys_count: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct LangSysRecord {
    tag: Tag,
    lang_sys_offset: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct LangSysTable {
    lookup_order: u16,
    required_feature_index: u16,
    feature_index_count: u16
}

type FeatureIndexToTag = HashMap<u16, FeatureTag>;
type TagToFeatureIndex = HashMap<FeatureTag, u16>;

macro_rules! try_as_u16 {
    ($val:expr, $scope:expr, $item:expr) => {{
        use std::convert::TryFrom;
        let val = $val;

        u16::try_from(val)
            .map_err(|_| $crate::compile_model::EncodeError::U16Overflow {
                scope: $scope,
                item: $item,
                value: val
            })
    }}
}

impl LangSys {
    fn ttf_decode(bytes: &[u8], feature_index_to_tag: &FeatureIndexToTag) -> DecodeResult<Self> {
        let table: LangSysTable = decode_from_slice(bytes);

        let required_feature =
            match table.required_feature_index {
                0xFFFF => None,
                idx => {
                    let tag = feature_index_to_tag.get(&idx)
                        .ok_or_else(||
                            DecodeError::UndefinedFeature("LangSys.required_feature_index", idx))?;

                    Some(tag.clone())
                }
            };

        decode_from_pool(table.feature_index_count, &bytes[LangSysTable::PACKED_LEN..])
            .map(|feature_index: u16| {
                Ok(feature_index_to_tag.get(&feature_index)
                    .ok_or_else(||
                        DecodeError::UndefinedFeature("LangSys.features", feature_index))?
                    .clone())
            })
            .collect::<DecodeResult<_>>()
            .map(|features| LangSys {
                required_feature,
                features,
            })
    }
}

impl LangSys {
    fn ttf_encode(&self, buf: &mut EncodeBuf, tag_to_feature_index: &TagToFeatureIndex) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let required_feature_index = self.required_feature
            .map(|tag| tag_to_feature_index.get(&tag)
                .ok_or_else(||
                    EncodeError::TagNotInFeatureList("LangSys.required_feature", tag.clone()))
                .map(|idx| *idx))
            .unwrap_or(Ok(0xFFFF))?;

        let table = LangSysTable {
            lookup_order: 0,
            required_feature_index,
            feature_index_count: try_as_u16!(self.features.len(),
                "LangSys".into(), "feature_index_count")?
        };

        buf.append(&table)?;

        for tag in &self.features {
            let idx = tag_to_feature_index.get(&tag)
                .ok_or_else(||
                    EncodeError::TagNotInFeatureList("LangSys.features", tag.clone()))?;

            buf.append(idx)?;
        }

        Ok(start)
    }
}

impl Script {
    fn ttf_decode(bytes: &[u8], feature_index_to_tag: &FeatureIndexToTag) -> DecodeResult<Self> {
        let table: ScriptTable = decode_from_slice(bytes);

        let lang_sys_records = decode_from_pool(
            table.lang_sys_count,
            &bytes[ScriptTable::PACKED_LEN..]);

        let default_lang_sys = LangSys::ttf_decode(&bytes[table.default_lang_sys as usize..],
            feature_index_to_tag)?;

        let lang_sys = lang_sys_records
            .map(|lsr: LangSysRecord|
                LangSys::ttf_decode(&bytes[lsr.lang_sys_offset as usize..], feature_index_to_tag)
                    .map(|sys| (lsr.tag, sys)))
            .collect::<DecodeResult<_>>()?;

        Ok(Script {
            default_lang_sys,
            lang_sys
        })
    }
}

impl Script {
    fn ttf_encode(&self, buf: &mut EncodeBuf, tag_to_feature_index: &TagToFeatureIndex) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.bytes.resize(start + ScriptTable::PACKED_LEN, 0u8);

        let table = ScriptTable {
            default_lang_sys: try_as_u16!(&self.default_lang_sys.ttf_encode(buf, tag_to_feature_index)? - start,
                "ScriptTable".into(), "default_lang_sys")?,
            lang_sys_count: try_as_u16!(self.lang_sys.len(),
                "ScriptTable".into(), "lang_sys_count")?
        };

        buf.encode_at(&table, start)?;

        for (tag, lang_sys) in &self.lang_sys {
            let record = LangSysRecord {
                tag: *tag,
                lang_sys_offset: try_as_u16!(lang_sys.ttf_encode(buf, tag_to_feature_index)? - start,
                    format!("LangSysRecord[{}]", tag), "lang_sys_offset")?
            };

            buf.append(&record)?;
        }

        Ok(start)
    }
}

impl ScriptList {
    #[inline]
    pub fn ttf_decode(bytes: &[u8], feature_list_bytes: &[u8]) -> DecodeResult<Self> {
        let records = decode_from_pool(decode_u16_be(bytes, 0), &bytes[2..]);

        // we need this so we can map feature indices to tags
        let feature_index_to_tag: FeatureIndexToTag = {
            let feature_records_count = decode_u16_be(feature_list_bytes, 0);

            decode_from_pool(feature_records_count, &feature_list_bytes[2..])
                .enumerate()
                .map(|(i, r): (_, FeatureRecord)| (i as u16, r.tag))
                .collect()
        };

        records
            .map(|sr: ScriptRecord| {
                let table_data = &bytes[sr.script_offset as usize..];

                Script::ttf_decode(table_data, &feature_index_to_tag)
                    .map(|script| (sr.tag, script))
            })
            .collect::<DecodeResult<HashMap<_, _>>>()
            .map(Self)
    }
}

impl ScriptList {
    pub fn ttf_encode(&self, buf: &mut EncodeBuf, feature_list: &FeatureList) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.append(
            &try_as_u16!(self.0.len(), "ScriptList".into(), "record_count")?)?;

        let mut record_offset = buf.bytes.len();
        buf.bytes.resize(record_offset +
            (self.0.len() * ScriptRecord::PACKED_LEN), 0u8);

        let tag_to_feature_index: TagToFeatureIndex = 
            feature_list.0.keys().enumerate()
                .map(|(i, tag)| (tag.clone(), i as u16))
                .collect();

        let dflt = script_tag!(D,F,L,T);

        if let Some(script) = self.0.get(&dflt) {
            let record = ScriptRecord {
                tag: dflt,
                script_offset: try_as_u16!(script.ttf_encode(buf, &tag_to_feature_index)? - start,
                    "ScriptList[DFLT]".into(), "script_offset")?
            };

            buf.encode_at(&record, record_offset)?;
            record_offset += ScriptRecord::PACKED_LEN;
        }

        for (tag, script) in &self.0 {
            if tag == &dflt {
                continue
            }

            let record = ScriptRecord {
                tag: *tag,
                script_offset: try_as_u16!(script.ttf_encode(buf, &tag_to_feature_index)? - start,
                    format!("ScriptList[{}]", tag), "script_offset")?
            };

            buf.encode_at(&record, record_offset)?;
            record_offset += ScriptRecord::PACKED_LEN;
        }

        Ok(start)
    }
}
