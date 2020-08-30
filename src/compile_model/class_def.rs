use std::ops;
use std::collections::BTreeSet;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::glyph_class::*;
use crate::glyph_order::*;

use crate::compile_model::util::encode::*;


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

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
struct Format1Header {
    format: u16,
    start_glyph_id: u16,
    glyph_count: u16
}

impl ClassDef {
    pub fn from_glyph_class(glyph_class: &GlyphClass, glyph_order: &GlyphOrder) -> Result<Self, GlyphOrderError> {
        let glyphs = glyph_class.iter_glyphs(glyph_order);

        glyphs.collect::<Result<_, GlyphOrderError>>()
            .map(Self)
    }

    fn encode_format_1(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        let header = Format1Header {
            format: 1,
            start_glyph_id: self.0.iter().next().map(|x| *x).unwrap_or(0u16),
            glyph_count: self.0.len() as u16
        };

        buf.append(&header)?;

        for id in self.0.iter() {
            buf.append(id)?;
        }

        Ok(start)
    }
}

impl TTFEncode for ClassDef {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        self.encode_format_1(buf)
    }
}
