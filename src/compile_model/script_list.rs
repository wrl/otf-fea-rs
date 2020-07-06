use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::{
    TTFEncode,
    TTFDecode,
    TTFTagged,
    EncodeBuf
};

use crate::parse_model as pm;

#[derive(Debug)]
pub struct ScriptList(Vec<TTFTagged<Script>>);

#[derive(Debug)]
pub struct Script {
    // FIXME: different types for untagged (default) and tagged language systems?
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
    tag: pm::Tag,
    script_offset: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct ScriptTable {
    default_lang_sys: u16,
    lang_sys_count: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct LangSysRecord {
    tag: pm::Tag,
    lang_sys_offset: u16
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct LangSysTable {
    lookup_order: u16,
    required_feature_index: u16,
    feature_index_count: u16
}

impl TTFDecode for LangSys {
    fn ttf_decode(bytes: &[u8]) -> Self {
        let table: LangSysTable = decode_from_slice(bytes);

        let required_feature_index =
            match table.required_feature_index {
                0xFFFF => None,
                otherwise => Some(otherwise)
            };

        let feature_indices = decode_from_pool(
            table.feature_index_count,
            &bytes[LangSysTable::PACKED_LEN..]);

        LangSys {
            required_feature_index,
            feature_indices: feature_indices.collect()
        }
    }
}

impl TTFEncode for LangSys {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> Result<usize, ()> {
        let start = buf.bytes.len();

        let table = LangSysTable {
            lookup_order: 0,
            required_feature_index: self.required_feature_index.unwrap_or(0xFFFF),
            feature_index_count: self.feature_indices.len() as u16
        };

        buf.append(&table)?;

        for idx in &self.feature_indices {
            buf.append(idx)?;
        }

        Ok(start)
    }
}

impl TTFDecode for Script {
    fn ttf_decode(bytes: &[u8]) -> Self {
        let table: ScriptTable = decode_from_slice(bytes);

        let lang_sys_records = decode_from_pool(
            table.lang_sys_count,
            &bytes[ScriptTable::PACKED_LEN..]);

        let default_lang_sys = LangSys::ttf_decode(
            &bytes[table.default_lang_sys as usize..]);

        let lang_sys = lang_sys_records.map(|lsr: LangSysRecord| {
            TTFTagged::new(lsr.tag,
                LangSys::ttf_decode(&bytes[lsr.lang_sys_offset as usize..]))
        });

        Script {
            default_lang_sys,
            lang_sys: lang_sys.collect()
        }
    }
}

impl TTFEncode for Script {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> Result<usize, ()> {
        let start = buf.bytes.len();

        buf.bytes.resize(start + ScriptTable::PACKED_LEN, 0u8);

        let table = ScriptTable {
            default_lang_sys: (buf.append(&self.default_lang_sys)? - start) as u16,
            lang_sys_count: self.lang_sys.len() as u16
        };

        buf.encode_at(&table, start)?;

        for TTFTagged(tag, lang_sys) in &self.lang_sys {
            let record = LangSysRecord {
                tag: *tag,
                lang_sys_offset: (buf.append(lang_sys)? - start) as u16
            };

            buf.append(&record)?;
        }

        Ok(start)
    }
}

impl ScriptList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let records = decode_from_pool(decode_u16_be(bytes, 0), &bytes[2..]);

        let scripts = records.map(|sr: ScriptRecord| {
            let table_data = &bytes[sr.script_offset as usize..];

            TTFTagged::new(sr.tag, Script::ttf_decode(table_data))
        });

        Self(scripts.collect())
    }
}

impl TTFEncode for ScriptList {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> Result<usize, ()> {
        let start = buf.bytes.len();

        buf.append(&(self.0.len() as u16))?;

        let mut record_offset = buf.bytes.len();
        buf.bytes.resize(record_offset +
            (self.0.len() * ScriptRecord::PACKED_LEN), 0u8);

        for TTFTagged(tag, script) in &self.0 {
            let record = ScriptRecord {
                tag: *tag,
                script_offset: (buf.append(script)? - start) as u16
            };

           buf.encode_at(&record, record_offset)?;
           record_offset += ScriptRecord::PACKED_LEN;
        }

        Ok(start)
    }
}
