use std::collections::{
    HashSet,
    HashMap
};

use thiserror::Error;
use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::class_def::*;
use crate::compile_model::coverage::*;
use crate::compile_model::error::*;


#[derive(Debug)]
pub struct PairClassIntersect(pub ValueRecord, pub ValueRecord);

#[derive(Debug, Default)]
pub struct PairClass {
    pub glyphs: (ClassDef, ClassDef),
    pub classes: (HashSet<ClassDef>, HashSet<ClassDef>),

    pub pairs: HashMap<(ClassDef, ClassDef), PairClassIntersect>
}

#[derive(Debug, Error)]
pub enum PairClassError {
    #[error("adding class pair resulted in a partial glyph class overlap. the subtable must be rebuilt.")]
    PartialOverlap
}


impl PairClass {
    pub fn can_add_pair(&self, pair: &(ClassDef, ClassDef)) -> bool {
        let glyphs_disjoint = (
            self.glyphs.0.is_disjoint(&pair.0),
            self.glyphs.1.is_disjoint(&pair.1)
        );

        if glyphs_disjoint == (true, true) {
            return true
        }

        let classes_present = (
            self.classes.0.contains(&pair.0),
            self.classes.1.contains(&pair.1)
        );

        if (!glyphs_disjoint.0 && classes_present.0) && (!glyphs_disjoint.1 && classes_present.1) {
            return true
        }

        false
    }

    pub fn add_pair(&mut self, pair: (ClassDef, ClassDef), value_records: (ValueRecord, ValueRecord))
            -> Result<(), PairClassError> {

        let glyphs_overlap = (
            pair.0.iter()
                .map(|g| self.glyphs.0.insert(*g))
                .fold(false, |acc, overlap| acc || overlap),

            pair.1.iter()
                .map(|g| self.glyphs.1.insert(*g))
                .fold(false, |acc, overlap| acc || overlap),
        );

        let classes_present = (
            self.classes.0.insert(pair.0.clone()),
            self.classes.1.insert(pair.1.clone())
        );

        if (glyphs_overlap.0 && !classes_present.0) || (glyphs_overlap.1 && !classes_present.1) {
            return Err(PairClassError::PartialOverlap);
        }

        self.pairs.insert(pair,
            PairClassIntersect(value_records.0, value_records.1));

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
        let mut coverage: CoverageLookup<()> = CoverageLookup::new();

        let mut classes = (
            self.classes.0.iter()
                .inspect(|cls| {
                    for glyph in cls.iter() {
                        coverage.insert(*glyph, ());
                    }
                })
                .collect::<Vec<_>>(),

            self.classes.1.iter().collect::<Vec<_>>()
        );

        classes.0.sort_by(|a, b|
            b.smallest_encoded_size()
                .cmp(&a.smallest_encoded_size()));

        let value_formats = self.pairs.values()
            .fold((0u16, 0u16), |vr, pair| {
                (vr.0 | pair.0.smallest_possible_format(),
                    vr.1 | pair.1.smallest_possible_format())
            });

        let start = buf.bytes.len();
        let null_vr = ValueRecord::zero();

        buf.defer_header_encode(
            |buf| Ok(PairPosFormat2Header {
                format: 2,
                coverage_offset: (buf.append(&coverage)? - start) as u16,

                value_format_1: value_formats.0,
                value_format_2: value_formats.1,

                class_def_1_offset: (classes.0.ttf_encode(buf, true)? - start) as u16,
                class_def_2_offset: (classes.1.ttf_encode(buf, false)? - start) as u16,

                class_1_count: classes.0.len() as u16,
                class_2_count: (classes.1.len() + 1) as u16
            }),

            |buf| {
                for x in &classes.0 {
                    // class 2 id 0
                    null_vr.encode_to_format(buf, value_formats.0, start)?;
                    null_vr.encode_to_format(buf, value_formats.1, start)?;

                    for y in &classes.1 {
                        // FIXME: clone. why the hell?
                        let intersect = match self.pairs.get(&((*x).clone(), (*y).clone())) {
                            Some(PairClassIntersect(a, b)) => (a, b),
                            None => (&null_vr, &null_vr)
                        };

                        intersect.0.encode_to_format(buf, value_formats.0, start)?;
                        intersect.1.encode_to_format(buf, value_formats.1, start)?;
                    }
                }

                Ok(())
            })
    }
}
