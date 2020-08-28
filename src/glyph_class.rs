use std::fmt;
use std::iter;

use crate::glyph_order::*;
use crate::glyph::*;
use crate::util::*;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum GlyphClassItem {
    Single(GlyphRef),

    Range {
        start: GlyphRef,
        end: GlyphRef
    },

    ClassRef(GlyphClassName)
}

impl From<GlyphRef> for GlyphClassItem {
    #[inline]
    fn from(glyph: GlyphRef) -> GlyphClassItem {
        GlyphClassItem::Single(glyph)
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GlyphClass(pub Vec<GlyphClassItem>);

impl GlyphClass {
    #[inline]
    pub fn from_single(glyph: GlyphRef) -> Self {
        Self(vec![glyph.into()])
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    #[inline]
    pub fn is_single(&self) -> bool {
        if let &[GlyphClassItem::Single(_)] = self.0.as_slice() {
            true
        } else {
            false
        }
    }

    pub fn iter_glyphs<'a>(&'a self, glyph_order: &'a GlyphOrder)
            -> impl Iterator<Item = Result<u16, GlyphOrderError>> + 'a {
        use GlyphClassItem::*;

        self.0.iter()
            .flat_map(move |i: &GlyphClassItem|
                match i {
                    Single(glyph) => {
                        Either2::A(iter::once(
                            glyph_order.id_for_glyph(glyph)
                                .ok_or_else(|| GlyphOrderError::UnknownGlyph(glyph.clone()))
                        ))
                    },

                    Range { start, end } => {
                        let start = match glyph_order.id_for_glyph(start) {
                            Some(id) => id,
                            None => return Either2::A(iter::once(
                                    Err(GlyphOrderError::UnknownGlyph(start.clone()))
                            ))
                        };

                        let end = match glyph_order.id_for_glyph(end) {
                            Some(id) => id,
                            None => return Either2::A(iter::once(
                                    Err(GlyphOrderError::UnknownGlyph(end.clone()))
                            ))
                        };

                        Either2::B((start..end+1).map(Ok))
                    },

                    ClassRef(_) => panic!()
                }
        )
    }
}

/////////////////////////
// named glyph classes
/////////////////////////

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
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
