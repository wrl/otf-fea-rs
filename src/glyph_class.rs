use std::fmt;
use std::iter;
use std::collections::HashMap;

use crate::glyph_order::*;
use crate::compile_model::*;
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
    pub fn into_single(mut self) -> Option<GlyphRef> {
        if !self.is_single() {
            return None
        }

        if let GlyphClassItem::Single(r) = self.0.pop().unwrap() {
            Some(r)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_single(&self) -> Option<&GlyphRef> {
        if let &[GlyphClassItem::Single(ref r)] = self.0.as_slice() {
            Some(r)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_single(&self) -> bool {
        self.as_single().is_some()
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
                        ))
                    },

                    Range { start, end } => {
                        let start = match glyph_order.id_for_glyph(start) {
                            Ok(id) => id,
                            e => return Either2::A(iter::once(e))
                        };

                        let end = match glyph_order.id_for_glyph(end) {
                            Ok(id) => id,
                            e => return Either2::A(iter::once(e))
                        };

                        Either2::B((start..end+1).map(Ok))
                    },

                    ClassRef(_) => panic!()
                }
        )
    }

    pub fn iter_glyphs_lookup<'a>(&'a self, glyph_order: &'a GlyphOrder,
            gc_table: &'a HashMap<GlyphClassName, GlyphClass>)
            -> impl Iterator<Item = Result<u16, CompileError>> + 'a {
        use GlyphClassItem::*;

        self.0.iter()
            .flat_map(move |i: &GlyphClassItem|
                match i {
                    Single(glyph) => {
                        Either3::A(iter::once(
                            glyph_order.id_for_glyph(glyph)
                                .map_err(|e| e.into())
                        ))
                    },

                    Range { start, end } => {
                        let start = match glyph_order.id_for_glyph(start) {
                            Ok(id) => id,
                            Err(e) => return Either3::A(iter::once(Err(e.into())))
                        };

                        let end = match glyph_order.id_for_glyph(end) {
                            Ok(id) => id,
                            Err(e) => return Either3::A(iter::once(Err(e.into())))
                        };

                        Either3::B((start..end+1).map(Ok))
                    },

                    ClassRef(name) => match gc_table.get(name) {
                        // FIXME: recursive glyph classes
                        Some(gc) => Either3::C(
                            gc.iter_glyphs(glyph_order)
                                .map(|r| r.map_err(|e| e.into()))
                        ),

                        None => Either3::A(iter::once(Err(
                            CompileError::UnknownGlyphClass(name.into())
                        )))
                    }
                }
        )
    }
}

/////////////////////////
// named glyph classes
/////////////////////////

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GlyphClassName(pub GlyphNameStorage);

impl From<&GlyphClassName> for String {
    fn from(name: &GlyphClassName) -> String {
        name.0.as_str().into()
    }
}

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
