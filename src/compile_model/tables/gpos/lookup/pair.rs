use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::util::variant::*;


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

impl TTFEncode for Pair {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            Pair::Glyphs(pg) => pg.ttf_encode(buf),
            Pair::Class(pc) => pc.ttf_encode(buf)
        }
    }
}
