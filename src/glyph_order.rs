use std::collections::HashMap;

#[allow(dead_code)]
pub struct TooManyGlyphs;
pub struct GlyphOrder(HashMap<String, u16>);

pub trait GlyphOrderExt<T>: Iterator<Item = T>
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

impl<T, I> GlyphOrderExt<T> for I
    where T: ToString,
          I: Iterator<Item = T> + Sized
{}
