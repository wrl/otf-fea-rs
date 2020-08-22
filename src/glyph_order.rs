use std::collections::HashMap;

use crate::glyph::*;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TooManyGlyphs;
pub struct GlyphOrder(HashMap<GlyphRef, u16>);

impl GlyphOrder {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn id_for_glyph(&self, glyph: &GlyphRef) -> Option<u16> {
        self.0.get(glyph)
            .map(|x| *x)
    }
}

pub trait IntoGlyphOrder: Iterator<Item = GlyphRef> + Sized
{
    fn collect_into_glyph_order(self) -> Result<GlyphOrder, TooManyGlyphs> {
        let mut map = HashMap::new();

        for (idx, glyph) in self.enumerate() {
            if idx > (u16::MAX as usize) {
                return Err(TooManyGlyphs);
            }

            map.insert(glyph, idx as u16);
        }

        Ok(GlyphOrder(map))
    }
}

impl<I> IntoGlyphOrder for I
    where I: Iterator<Item = GlyphRef> + Sized
{}
