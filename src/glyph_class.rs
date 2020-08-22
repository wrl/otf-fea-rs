use std::fmt;

use crate::glyph::*;

#[derive(Debug, Clone)]
pub enum GlyphClassItem {
    Single(GlyphRef),
    Range {
        start: GlyphRef,
        end: GlyphRef
    },
    ClassRef(GlyphClassName)
}

impl From<GlyphRef> for GlyphClassItem {
    fn from(glyph: GlyphRef) -> GlyphClassItem {
        GlyphClassItem::Single(glyph)
    }
}

#[derive(Debug, Clone)]
pub struct GlyphClass(pub Vec<GlyphClassItem>);

impl GlyphClass {
    pub fn from_single(glyph: GlyphRef) -> GlyphClass {
        GlyphClass(vec![glyph.into()])
    }
}

/////////////////////////
// named glyph classes
/////////////////////////

#[derive(Clone)]
pub struct GlyphClassName(pub GlyphNameStorage);

impl fmt::Debug for GlyphClassName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlyphClassName(\"@")?;

        for ch in &self.0 {
            write!(f, "{}", ch)?;
        }

        write!(f, "\")")
    }
}

#[derive(Debug)]
pub struct NamedGlyphClass {
    pub name: GlyphClassName,
    pub glyph_class: GlyphClass
}
