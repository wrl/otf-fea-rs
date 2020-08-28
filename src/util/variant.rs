#[macro_export]
macro_rules! impl_from_variant {
    ($to_type:ty, $from_type:ident, $variant:ident) => {
        impl From<$from_type> for $to_type {
            fn from(x: $from_type) -> $to_type {
                <$to_type>::$variant(x)
            }
        }
    };

    ($to_type:ty, $from_type:ident) => {
        $crate::impl_from_variant!($to_type, $from_type, $from_type);
    };
}

pub trait VariantExt<E>: Sized {
    fn get_variant(_: &E) -> Option<&Self>;
    fn get_variant_mut(_: &mut E) -> Option<&mut Self>;
}

#[macro_export]
macro_rules! impl_variant_ext_for {
    ($enum:ident, $variant:ident, $ty:ident) => {
        impl VariantExt<$enum> for $ty {
            #[inline]
            fn get_variant(en: &$enum) -> Option<&$ty> {
                match en {
                    $enum::$variant(l) => Some(l),
                    _ => None
                }
            }

            #[inline]
            fn get_variant_mut(en: &mut $enum) -> Option<&mut $ty> {
                match en {
                    $enum::$variant(l) => Some(l),
                    _ => None
                }
            }
        }

        $crate::impl_from_variant!($enum, $ty, $variant);
    };
}
