use crate::util::variant::*;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::lookup::*;

use super::pair_glyphs::*;
use super::pair_class::*;


#[derive(Debug)]
pub enum Pair {
    Glyphs(PairGlyphs),
    Class(PairClass)
}

crate::impl_variant_ext_for!(Pair, Glyphs, PairGlyphs);
crate::impl_variant_ext_for!(Pair, Class, PairClass);

impl TTFDecode for Pair {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let format = decode_u16_be(bytes, 0);

        match format {
            1 => PairGlyphs::ttf_decode(bytes).map(Pair::Glyphs),
            _ => Err(DecodeError::InvalidValue("format", "PairPos".into()))
        }
    }
}

pub enum PairSubtableEncoder<'a> {
    Glyphs(PairGlyphsSplittingEncoder<'a>),
    Class(SingularSubtableEncoder<'a, PairClass>)
}

impl<'a> TTFSubtableEncoder<'a> for PairSubtableEncoder<'a> {
    #[inline]
    fn encode_next_subtable(&mut self, buf: &mut EncodeBuf) -> Option<EncodeResult<usize>> {
        use PairSubtableEncoder::*;

        match self {
            Glyphs(e) => e.encode_next_subtable(buf),
            Class(e) => e.encode_next_subtable(buf)
        }
    }
}

impl<'a> TTFSubtableEncode<'a> for Pair {
    type Encoder = PairSubtableEncoder<'a>;

    #[inline]
    fn ttf_subtable_encoder(&'a self) -> Self::Encoder {
        match self {
            Pair::Glyphs(pg) => PairSubtableEncoder::Glyphs(pg.ttf_subtable_encoder()),
            Pair::Class(pg) => PairSubtableEncoder::Class(pg.ttf_subtable_encoder())
        }
    }
}
