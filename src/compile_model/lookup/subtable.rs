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
}
