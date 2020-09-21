// use endian_codec::{PackedSize, EncodeBE, DecodeBE};
// 
// use crate::compile_model::util::encode::*;
use crate::compile_model::tables::gpos::*;
use crate::compile_model::coverage::*;


#[derive(Debug)]
pub struct Anchors {
    entry: Anchor,
    exit: Anchor
}

#[derive(Debug, Default)]
pub struct Cursive(pub CoverageLookup<Anchors>);

impl Cursive {
    pub fn add_rule(&mut self, glyph_id: u16, entry: Anchor, exit: Anchor) {
        self.0.insert(glyph_id, Anchors {
            entry,
            exit
        });
    }
}
