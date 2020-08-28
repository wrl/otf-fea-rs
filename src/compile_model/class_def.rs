use std::ops;
use std::collections::BTreeSet;

use crate::glyph_class::*;
use crate::glyph_order::*;


#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ClassDef(pub BTreeSet<u16>);


impl Default for ClassDef {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl ops::Deref for ClassDef {
    type Target = BTreeSet<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for ClassDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl ClassDef {
    pub fn from_glyph_class(glyph_class: &GlyphClass, glyph_order: &GlyphOrder) -> Result<Self, GlyphOrderError> {
        let glyphs = glyph_class.iter_glyphs(glyph_order);

        glyphs.collect::<Result<_, GlyphOrderError>>()
            .map(Self)
    }
}
