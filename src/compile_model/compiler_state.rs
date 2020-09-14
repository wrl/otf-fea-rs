use std::collections::HashMap;

use crate::{
    GlyphOrder,
    Tag
};

use crate::glyph_class::*;

use crate::parse_model::{
    MarkClassName,
    Anchor
};

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
    pub mark_class_table: HashMap<MarkClassName, MarkClassData>,

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

            tables_encoded: Vec::new(),
        }
    }
}
