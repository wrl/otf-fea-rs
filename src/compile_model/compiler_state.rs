use std::collections::HashMap;
use std::convert::TryInto;

use crate::GlyphOrder;

use crate::glyph_class::*;

use crate::compile_model::error::*;
use crate::parse_model as pm;

use super::tables::gpos::Anchor;
use super::tables;


pub type MarkClassGlyphClass = (GlyphClass, Anchor);
pub type MarkClassData = Vec<MarkClassGlyphClass>;
pub type NamedGlyphClassTable = HashMap<GlyphClassName, GlyphClass>;

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
    pub glyph_class_table: NamedGlyphClassTable,
}

pub struct CompilerOutput {
    pub glyph_order: GlyphOrder,

    pub head: Option<tables::Head>,
    pub gpos: Option<tables::GPOS>,
    pub gsub: Option<tables::GSUB>
}

impl From<CompilerState> for CompilerOutput {
    fn from(state: CompilerState) -> Self {
        CompilerOutput {
            glyph_order: state.glyph_order,

            head: state.head,
            gpos: state.gpos,
            gsub: state.gsub,
        }
    }
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
            glyph_class_table: HashMap::new(),
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
