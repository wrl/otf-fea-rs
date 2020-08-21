use std::collections::HashMap;
use ascii::AsAsciiStr;

use crate::parse_model::GlyphRef;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TooManyGlyphs;
pub struct GlyphOrder(HashMap<String, u16>);

impl GlyphOrder {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn id_for_glyph(&self, glyph: &GlyphRef) -> Option<u16> {
        match glyph {
            GlyphRef::Name(name) => {
                let s = name.0.as_ascii_str().unwrap().as_str();
                self.0.get(s)
                    .map(|x| *x)
            },

            GlyphRef::CID(_) => None
        }
    }
}

pub trait IntoGlyphOrder<T>: Iterator<Item = T>
    where T: ToString,
          Self: Sized
{
    fn collect_into_glyph_order(self) -> Result<GlyphOrder, TooManyGlyphs> {
        let mut map = HashMap::new();

        for (idx, glyph_name) in self.enumerate() {
            if idx > (u16::MAX as usize) {
                return Err(TooManyGlyphs);
            }

            map.insert(glyph_name.to_string(), idx as u16);
        }

        Ok(GlyphOrder(map))
    }
}

impl<T, I> IntoGlyphOrder<T> for I
    where T: ToString,
          I: Iterator<Item = T> + Sized
{}
