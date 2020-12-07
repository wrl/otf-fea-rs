use std::iter;

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

pub trait TTFSubtableEncode<'a, 'buf:'a> {
    type Iter: Iterator<Item = EncodeResult<usize>>;

    fn ttf_subtable_encode(&'a self, buf: &'buf mut EncodeBuf) -> Self::Iter;
}

impl<'a, 'buf:'a, T: TTFEncode> TTFSubtableEncode<'a, 'buf> for T {
    type Iter = iter::Once<EncodeResult<usize>>;

    #[inline]
    fn ttf_subtable_encode(&self, buf: &mut EncodeBuf) -> Self::Iter
    {
        iter::once(self.ttf_encode(buf))
    }
}
