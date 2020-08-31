#![allow(non_camel_case_types)]

pub mod tables;

mod class_def;
pub use class_def::ClassDef;

mod coverage;
pub use coverage::CoverageLookup;

mod error;
pub use error::*;

mod feature_list;
pub use feature_list::{
    FeatureList,
    FeatureRecord,
};

mod lookup_list;
pub use lookup_list::{
    LookupList,
    LookupFlags,
    Lookup,

    LookupSubtable
};

mod script_list;
pub use script_list::{
    ScriptList,
    Script,
};

#[macro_use]
pub mod util;
pub use util::TTFVersion;

mod value_record;
pub use value_record::{
    ValueRecord,
    ValueRecordFromParsed
};

mod ttf_table;
pub use ttf_table::*;


use crate::{
    GlyphOrder,
    Tag
};

pub struct CompilerState {
    pub glyph_order: GlyphOrder,

    pub head_table: Option<tables::Head>,
    pub gpos_table: Option<tables::GPOS>,

    pub tables_encoded: Vec<(Tag, Vec<u8>)>
}

impl CompilerState {
    pub fn new() -> Self {
        Self {
            glyph_order: GlyphOrder::new(),

            head_table: None,
            gpos_table: None,

            tables_encoded: Vec::new(),
        }
    }
}
