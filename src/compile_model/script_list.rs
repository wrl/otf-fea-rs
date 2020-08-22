use std::collections::HashMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::{
    TTFTagged,
};

use crate::{
    Tag,
    tag
};

#[derive(Debug)]
pub struct ScriptList(HashMap<Tag, Script>);

impl ScriptList {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[inline]
    pub fn script_for_tag(&self, tag: &Tag) -> Option<&Script> {
        self.0.get(tag)
    }

    #[inline]
    pub fn script_for_tag_mut(&mut self, tag: &Tag) -> &mut Script {
        self.0.entry(*tag)
            .or_insert_with(|| Script {
                default_lang_sys: LangSys {
                    required_feature_index: None,
                    feature_indices: Vec::new()
                },

                lang_sys: Vec::new()
            })
    }
}

#[derive(Debug)]
pub struct Script {
    pub default_lang_sys: LangSys,
    pub lang_sys: Vec<TTFTagged<LangSys>>
}

#[derive(Debug)]
pub struct LangSys {
    pub required_feature_index: Option<u16>,
    pub feature_indices: Vec<u16>
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct ScriptRecord {
    tag: Tag,
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

impl TTFDecode for LangSys {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let table: LangSysTable = decode_from_slice(bytes);

        let required_feature_index =
            match table.required_feature_index {
                0xFFFF => None,
                otherwise => Some(otherwise)
            };

        let feature_indices = decode_from_pool(
            table.feature_index_count,
            &bytes[LangSysTable::PACKED_LEN..]);

        Ok(LangSys {
            required_feature_index,
            feature_indices: feature_indices.collect()
        })
    }
}

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

impl TTFEncode for LangSys {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let table = LangSysTable {
            lookup_order: 0,
            required_feature_index: self.required_feature_index.unwrap_or(0xFFFF),
            feature_index_count: try_as_u16!(self.feature_indices.len(),
                "LangSys".into(), "feature_index_count")?
        };

        buf.append(&table)?;

        for idx in &self.feature_indices {
            buf.append(idx)?;
        }

        Ok(start)
    }
}

impl TTFDecode for Script {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let table: ScriptTable = decode_from_slice(bytes);

        let lang_sys_records = decode_from_pool(
            table.lang_sys_count,
            &bytes[ScriptTable::PACKED_LEN..]);

        let default_lang_sys = LangSys::ttf_decode(
            &bytes[table.default_lang_sys as usize..])?;

        let lang_sys = lang_sys_records
            .map(|lsr: LangSysRecord|
                LangSys::ttf_decode(&bytes[lsr.lang_sys_offset as usize..])
                    .map(|sys| TTFTagged::new(lsr.tag, sys)))
            .collect::<DecodeResult<_>>()?;

        Ok(Script {
            default_lang_sys,
            lang_sys
        })
    }
}

impl TTFEncode for Script {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.bytes.resize(start + ScriptTable::PACKED_LEN, 0u8);

        let table = ScriptTable {
            default_lang_sys: try_as_u16!(buf.append(&self.default_lang_sys)? - start,
                "ScriptTable".into(), "default_lang_sys")?,
            lang_sys_count: try_as_u16!(self.lang_sys.len(),
                "ScriptTable".into(), "lang_sys_count")?
        };

        buf.encode_at(&table, start)?;

        for TTFTagged(tag, lang_sys) in &self.lang_sys {
            let record = LangSysRecord {
                tag: *tag,
                lang_sys_offset: try_as_u16!(buf.append(lang_sys)? - start,
                    format!("LangSysRecord[{}]", tag), "lang_sys_offset")?
            };

            buf.append(&record)?;
        }

        Ok(start)
    }
}

impl TTFDecode for ScriptList {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let records = decode_from_pool(decode_u16_be(bytes, 0), &bytes[2..]);

        records
            .map(|sr: ScriptRecord| {
                let table_data = &bytes[sr.script_offset as usize..];

                Script::ttf_decode(table_data)
                    .map(|script| (sr.tag, script))
            })
            .collect::<DecodeResult<HashMap<_, _>>>()
            .map(Self)
    }
}

impl TTFEncode for ScriptList {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.append(
            &try_as_u16!(self.0.len(), "ScriptList".into(), "record_count")?)?;

        let mut record_offset = buf.bytes.len();
        buf.bytes.resize(record_offset +
            (self.0.len() * ScriptRecord::PACKED_LEN), 0u8);

        let dflt = tag!(D,F,L,T);

        if let Some(script) = self.0.get(&dflt) {
            let record = ScriptRecord {
                tag: dflt,
                script_offset: try_as_u16!(buf.append(script)? - start,
                format!("ScriptList[{}]", dflt), "script_offset")?
            };

            buf.encode_at(&record, record_offset)?;
            record_offset += ScriptRecord::PACKED_LEN;
        }

        for (tag, script) in &self.0 {
            if *tag == tag!(D,F,L,T) {
                continue
            }

            let record = ScriptRecord {
                tag: *tag,
                script_offset: try_as_u16!(buf.append(script)? - start,
                format!("ScriptList[{}]", tag), "script_offset")?
            };

            buf.encode_at(&record, record_offset)?;
            record_offset += ScriptRecord::PACKED_LEN;
        }

        Ok(start)
    }
}
