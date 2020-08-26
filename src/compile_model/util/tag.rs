use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::*;

macro_rules! impl_for {
    ($type:ty) => {
        impl PackedSize for $type {
            const PACKED_LEN: usize = 4;
        }

        impl EncodeBE for $type {
            #[inline]
            fn encode_as_be_bytes(&self, bytes: &mut [u8]) {
                let repr:u32 =
                    (self.0[0].as_byte() as u32) << 24
                    | (self.0[1].as_byte() as u32) << 16
                    | (self.0[2].as_byte() as u32) << 8
                    | (self.0[3].as_byte() as u32);

                bytes.copy_from_slice(&(repr.to_be_bytes()));
            }
        }

        impl DecodeBE for $type {
            #[inline]
            fn decode_from_be_bytes(bytes: &[u8]) -> Self {
                let mut arr = [0u8; 4];
                arr.copy_from_slice(bytes);

                // FIXME: can't be unwrap()ing
                <$type>::from_bytes(&arr).unwrap()
            }
        }
    }
}

impl_for!(Tag);
impl_for!(FeatureTag);
impl_for!(ScriptTag);
