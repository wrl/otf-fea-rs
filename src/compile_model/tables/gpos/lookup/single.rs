use crate::compile_model::util::encode::*;
use crate::util::variant::*;


use super::single_class::*;
use super::single_array::*;


#[derive(Debug)]
pub enum Single {
    Class(SingleClass),
    Array(SingleArray)
}

crate::impl_variant_ext_for!(Single, Array, SingleArray);
crate::impl_variant_ext_for!(Single, Class, SingleClass);

impl TTFEncode for Single {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            Single::Class(sc) => sc.ttf_encode(buf),
            Single::Array(sa) => sa.ttf_encode(buf),
        }
    }
}
