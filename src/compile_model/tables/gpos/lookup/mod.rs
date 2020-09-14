use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::lookup::*;
use crate::util::variant::*;


mod pair_glyphs;
pub use pair_glyphs::*;

mod pair_class;
pub use pair_class::*;

mod mark_to_base;
pub use mark_to_base::*;

mod mark_to_mark;
pub use mark_to_mark::*;


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

macro_rules! impl_subtable_for {
    ($ty:ident) => {
        $crate::impl_lookup_subtable_for!(GPOSLookup, $ty, $ty);
    }
}

#[derive(Debug)]
pub enum GPOSLookup {
    Pair(Lookup<Pair>),
    MarkToBase(Lookup<MarkToBase>),
    MarkToMark(Lookup<MarkToMark>),
}

impl_subtable_for!(Pair);
impl_subtable_for!(MarkToBase);
impl_subtable_for!(MarkToMark);

impl TTFDecode for GPOSLookup {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let lookup_type = decode_u16_be(bytes, 0);

        match lookup_type {
            2 => Lookup::ttf_decode(bytes).map(GPOSLookup::Pair),
            _ => Err(DecodeError::InvalidValue("lookup_type", "GPOS Lookup".into()))
        }
    }
}

impl TTFEncode for GPOSLookup {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            GPOSLookup::Pair(lookup) => lookup.ttf_encode_with_lookup_type(buf, 2),
            l => panic!("unimplemented encode for {:?}", l)
        }
    }
}
