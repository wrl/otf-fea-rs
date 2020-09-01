use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::lookup::*;


mod multiple;
pub use multiple::*;


macro_rules! impl_subtable_for {
    ($ty:ident) => {
        $crate::impl_lookup_subtable_for!(GSUBLookup, $ty, $ty);
    }
}

#[derive(Debug)]
pub enum GSUBLookup {
    Multiple(Lookup<Multiple>)
}

impl_subtable_for!(Multiple);


impl TTFDecode for GSUBLookup {
    fn ttf_decode(_bytes: &[u8]) -> DecodeResult<Self> {
        // let lookup_type = decode_u16_be(bytes, 0);

        panic!("unimplemented");
    }
}

impl TTFEncode for GSUBLookup {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            GSUBLookup::Multiple(lookup) => lookup.ttf_encode_with_lookup_type(buf, 2)
        }
    }
}
