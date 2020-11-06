use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::lookup::*;


mod single_array;
mod single_class;
mod single;
pub use single_array::*;
pub use single_class::*;
pub use single::*;

mod pair_glyphs;
mod pair_class;
mod pair;
pub use pair_glyphs::*;
pub use pair_class::*;
pub use pair::*;

pub mod cursive;
pub use cursive::Cursive;

mod mark_to_base;
pub use mark_to_base::*;

mod mark_to_mark;
pub use mark_to_mark::*;


macro_rules! impl_subtable_for {
    ($ty:ident) => {
        $crate::impl_lookup_subtable_for!(GPOSLookup, $ty, $ty);
    }
}

#[derive(Debug)]
pub enum GPOSLookup {
    Single(Lookup<Single>),
    SingleArray(Lookup<SingleArray>),
    Pair(Lookup<Pair>),
    Cursive(Lookup<Cursive>),
    MarkToBase(Lookup<MarkToBase>),
    MarkToMark(Lookup<MarkToMark>),
}

impl_subtable_for!(Single);
impl_subtable_for!(SingleArray);
impl_subtable_for!(Pair);
impl_subtable_for!(Cursive);
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
            GPOSLookup::Single(lookup) => lookup.ttf_encode_with_lookup_type(buf, 1),
            GPOSLookup::SingleArray(lookup) => lookup.ttf_encode_with_lookup_type(buf, 1),

            GPOSLookup::Pair(lookup) => lookup.ttf_encode_with_lookup_type(buf, 2),
            GPOSLookup::Cursive(lookup) => lookup.ttf_encode_with_lookup_type(buf, 3),
            GPOSLookup::MarkToBase(lookup) => lookup.ttf_encode_with_lookup_type(buf, 4),
            l => panic!("unimplemented encode for {:?}", l)
        }
    }
}
