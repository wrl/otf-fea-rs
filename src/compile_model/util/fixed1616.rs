use endian_codec::{PackedSize, EncodeBE, DecodeBE};
use fixed::{types::I16F16};

#[derive(Debug, Copy, Clone)]
pub struct Fixed1616(I16F16);

impl Fixed1616 {
    pub const fn from_bits(bits: i32) -> Self {
        Self(I16F16::from_bits(bits))
    }

    pub const fn to_bits(self) -> i32 {
        self.0.to_bits()
    }

    pub fn from_f32(src: f32) -> Self {
        Self(I16F16::from_num(src))
    }

    pub fn to_f32(self) -> f32 {
        self.0.to_num()
    }
}

impl PackedSize for Fixed1616 {
    const PACKED_LEN: usize = 4;
}

impl EncodeBE for Fixed1616 {
    #[inline]
    fn encode_as_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&(self.0.to_be_bytes()));
    }
}

impl DecodeBE for Fixed1616 {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Self(I16F16::from_be_bytes(arr))
    }
}
