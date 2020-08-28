use std::ops;
use std::collections::BTreeMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::value_record::*;
use crate::compile_model::class_def::*;


#[derive(Debug)]
pub struct PairClassIntersect(pub ValueRecord, pub ValueRecord);

type PairClassStorage = BTreeMap<ClassDef, BTreeMap<ClassDef, Vec<PairClassIntersect>>>;

#[derive(Debug)]
pub struct PairClass(pub PairClassStorage);


impl Default for PairClass {
    fn default() -> Self {
        PairClass(BTreeMap::new())
    }
}

impl ops::Deref for PairClass {
    type Target = PairClassStorage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for PairClass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
