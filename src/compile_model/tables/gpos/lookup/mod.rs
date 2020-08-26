use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::lookup_list::*;


mod pair_glyphs;
pub use pair_glyphs::*;

mod pair_class;
pub use pair_class::*;


#[derive(Debug)]
pub enum GPOSLookup {
    PairGlyphs(Lookup<PairGlyphs>)
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

impl_subtable_for!(PairGlyphs);

impl TTFDecode for GPOSLookup {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let lookup_type = decode_u16_be(bytes, 0);

        match lookup_type {
            2 => Lookup::ttf_decode(bytes).map(GPOSLookup::PairGlyphs),
            _ => Err(DecodeError::InvalidValue("lookup_type", "GPOS Lookup".into()))
        }
    }
}

impl TTFEncode for GPOSLookup {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            GPOSLookup::PairGlyphs(lookup) => lookup.ttf_encode(buf, 2)
        }
    }
}
