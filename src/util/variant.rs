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
