use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TooManyGlyphs;
pub struct GlyphOrder(HashMap<String, u16>);

impl GlyphOrder {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
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
