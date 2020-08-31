use std::collections::BTreeMap;

use thiserror::Error;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::class_def::*;
use crate::compile_model::error::*;


#[derive(Debug)]
pub struct PairClassIntersect(pub ValueRecord, pub ValueRecord);

type PairClassStorage = BTreeMap<ClassDef, BTreeMap<ClassDef, Vec<PairClassIntersect>>>;

#[derive(Debug)]
pub struct PairClass(pub PairClassStorage);

#[derive(Debug, Error)]
pub enum PairClassError {
    #[error("adding pair would result in a partial glyph class overlap")]
    PartialOverlap
}


impl Default for PairClass {
    fn default() -> Self {
        PairClass(BTreeMap::new())
    }
}

impl PairClass {
    pub fn can_add_pair(&self, _pair: &(ClassDef, ClassDef)) -> bool {
        true
    }

    pub fn add_pair(&mut self, pair: (ClassDef, ClassDef), value_records: (ValueRecord, ValueRecord))
            -> Result<(), PairClassError> {
        let first_class = self.0.entry(pair.0)
            .or_default();

        let second_class = first_class.entry(pair.1)
            .or_default();

        second_class.push(PairClassIntersect(
            value_records.0,
            value_records.1
        ));

        Ok(())
    }
}


#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct PairPosFormat2Header {
    pub format: u16,
    pub coverage_offset: u16,
    pub value_format_1: u16,
    pub value_format_2: u16,
    pub class_def_1_offset: u16,
    pub class_def_2_offset: u16,
    pub class_1_count: u16,
    pub class_2_count: u16
}

impl TTFEncode for PairClass {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.bytes.resize(start + PairPosFormat2Header::PACKED_LEN, 0u8);

        for _c1 in &self.0 {
        }

        Ok(start)
    }
}
