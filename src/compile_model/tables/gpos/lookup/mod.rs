use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::lookup_list::*;
use crate::util::variant::*;


mod pair_glyphs;
pub use pair_glyphs::*;

mod pair_class;
pub use pair_class::*;


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
            Pair::Glyphs(glyphs) => glyphs.ttf_encode(buf),
            _ => Ok(buf.bytes.len())
        }
    }
}

#[derive(Debug)]
pub enum GPOSLookup {
    Pair(Lookup<Pair>)
}

macro_rules! impl_subtable_for {
    ($lookup:ident, $ty:ty, $variant:ident) => {
        impl LookupSubtable<$lookup> for $ty {
            #[inline]
            fn new_lookup() -> $lookup {
                $lookup::$variant(Lookup::new())
            }

            #[inline]
            fn get_lookup_variant(lookup: &$lookup) -> Option<&Lookup<$ty>> {
                match lookup {
                    $lookup::$variant(l) => Some(l)
                }
            }

            #[inline]
            fn get_lookup_variant_mut(lookup: &mut $lookup) -> Option<&mut Lookup<$ty>> {
                match lookup {
                    $lookup::$variant(l) => Some(l)
                }
            }
        }
    };

    ($ty:ident) => {
        impl_subtable_for!(GPOSLookup, $ty, $ty);
    }
}

impl_subtable_for!(Pair);

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
            GPOSLookup::Pair(lookup) => lookup.ttf_encode(buf, 2)
        }
    }
}
