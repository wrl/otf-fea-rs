use std::collections::HashMap;
use std::convert::TryInto;

use crate::{
    GlyphOrder,
    Tag
};

use crate::glyph_class::*;

use crate::compile_model::error::*;
use crate::parse_model as pm;

use super::tables::gpos::Anchor;
use super::tables;


pub type MarkClassGlyphClass = (GlyphClass, Anchor);
pub type MarkClassData = Vec<MarkClassGlyphClass>;

pub struct CompilerState {
    pub glyph_order: GlyphOrder,

    pub head: Option<tables::Head>,
    pub gpos: Option<tables::GPOS>,
    pub gsub: Option<tables::GSUB>,

    // Note: All mark class definition statements must precede any use of a mark class in the
    // feature file. Once any position statement has referenced a mark class, no more mark
    // statements are allowed.
    pub mark_class_statements_allowed: bool,
    pub mark_class_table: HashMap<pm::MarkClassName, MarkClassData>,

    pub anchor_table: HashMap<pm::AnchorName, Anchor>,

    pub tables_encoded: Vec<(Tag, Vec<u8>)>
}

impl CompilerState {
    pub fn new() -> Self {
        Self {
            glyph_order: GlyphOrder::new(),

            head: None,
            gpos: None,
            gsub: None,

            mark_class_statements_allowed: true,
            mark_class_table: HashMap::new(),

            anchor_table: HashMap::new(),

            tables_encoded: Vec::new(),
        }
    }

    pub fn lookup_anchor(&self, parsed: &pm::Anchor) -> CompileResult<Anchor> {
        use pm::Anchor::*;

        match parsed {
            Named(name) => self.anchor_table.get(name)
                .map(|a| a.clone())
                .ok_or_else(|| CompileError::UndefinedReference("anchor", name.into())),

            anchor => anchor.try_into()
        }
    }
}
