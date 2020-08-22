use std::collections::HashMap;

use thiserror::Error;

use crate::glyph::*;


#[derive(Debug, Error)]
pub enum GlyphOrderError {
    #[error("tried to create a GlyphOrder with more than 65536 glyphs")]
    TooManyGlyphs,

    #[error(transparent)]
    GlyphError(#[from] GlyphError)
}

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

pub trait IntoGlyphOrder<T>: Iterator<Item = T> + Sized
{
    fn collect_into_glyph_order(self) -> Result<GlyphOrder, GlyphOrderError>;
}

impl<E, I> IntoGlyphOrder<Result<GlyphRef, E>> for I
    where I: Iterator<Item = Result<GlyphRef, E>> + Sized,
          E: Into<GlyphOrderError>
{
    fn collect_into_glyph_order(self) -> Result<GlyphOrder, GlyphOrderError> {
        let mut map = HashMap::new();

        for (idx, glyph) in self.enumerate() {
            if idx > (u16::MAX as usize) {
                return Err(GlyphOrderError::TooManyGlyphs);
            }

            map.insert(glyph.map_err(|e| e.into())?, idx as u16);
        }

        Ok(GlyphOrder(map))
    }
}

impl<I> IntoGlyphOrder<GlyphRef> for I
    where I: Iterator<Item = GlyphRef> + Sized
{
    fn collect_into_glyph_order(self) -> Result<GlyphOrder, GlyphOrderError> {
        let mut map = HashMap::new();

        for (idx, glyph) in self.enumerate() {
            if idx > (u16::MAX as usize) {
                return Err(GlyphOrderError::TooManyGlyphs);
            }

            map.insert(glyph, idx as u16);
        }

        Ok(GlyphOrder(map))
    }
}
