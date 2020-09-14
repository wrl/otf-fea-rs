use std::collections::HashMap;

use crate::compile_model::tables::gpos::*;
use crate::glyph_class::*;
use crate::glyph_order::*;

use crate::parse_model::MarkClassName;

#[derive(Debug, Default)]
pub struct MarkToMark {
    base_marks: HashMap<(u16, u16), Anchor>
}

impl MarkToMark {
    pub fn add_mark(&mut self, _glyph_order: &GlyphOrder, _base_mark: &GlyphClass, _mark: (&Anchor, &MarkClassName)) {
    }
}
