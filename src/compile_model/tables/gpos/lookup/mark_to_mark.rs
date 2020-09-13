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
    pub fn add_mark(&mut self, glyph_order: &GlyphOrder, base_mark: &GlyphClass, mark: (&Anchor, &MarkClassName)) {
    }
}
