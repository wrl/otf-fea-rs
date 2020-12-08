use crate::compile_model::util::encode::*;
use super::*;


pub trait LookupSubtable<E>: Sized {
    fn new_lookup() -> E;
    fn get_lookup_variant(_: &E) -> Option<&Lookup<Self>>;
    fn get_lookup_variant_mut(_: &mut E) -> Option<&mut Lookup<Self>>;
}

#[macro_export]
macro_rules! impl_lookup_subtable_for {
    ($lookup:ident, $ty:ty, $variant:ident) => {
        impl LookupSubtable<$lookup> for $ty {
            #[inline]
            fn new_lookup() -> $lookup {
                $lookup::$variant(Lookup::new())
            }

            #[allow(unreachable_patterns)]
            #[inline]
            fn get_lookup_variant(lookup: &$lookup) -> Option<&Lookup<$ty>> {
                match lookup {
                    $lookup::$variant(l) => Some(l),
                    _ => None
                }
            }

            #[allow(unreachable_patterns)]
            #[inline]
            fn get_lookup_variant_mut(lookup: &mut $lookup) -> Option<&mut Lookup<$ty>> {
                match lookup {
                    $lookup::$variant(l) => Some(l),
                    _ => None
                }
            }
        }
    };
}

pub trait TTFSubtableEncode<'a> {
    type Encoder: TTFSubtableEncoder<'a>;

    fn ttf_subtable_encoder(&'a self) -> Self::Encoder;
}

pub trait TTFSubtableEncoder<'a> {
    fn encode_next_subtable(&mut self, buf: &mut EncodeBuf) -> Option<EncodeResult<usize>>;
}

// blanket impl for subtables which don't support splitting
pub struct SingularSubtableEncoder<'a, T: TTFEncode> {
    subtable: Option<&'a T>
}

impl<'a, T: TTFEncode + 'a> TTFSubtableEncode<'a> for T {
    type Encoder = SingularSubtableEncoder<'a, T>;

    fn ttf_subtable_encoder(&'a self) -> Self::Encoder {
        SingularSubtableEncoder {
            subtable: Some(self)
        }
    }
}

impl<'a, T: TTFEncode> TTFSubtableEncoder<'a> for SingularSubtableEncoder<'a, T> {
    #[inline]
    fn encode_next_subtable(&mut self, buf: &mut EncodeBuf) -> Option<EncodeResult<usize>> {
        self.subtable.take()
            .map(|st| st.ttf_encode(buf))
    }
}
