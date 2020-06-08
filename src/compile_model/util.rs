use endian_codec::{PackedSize, EncodeBE, DecodeBE};
use fixed::{types::I16F16};

use chrono::{Utc, DateTime, TimeZone};

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

#[derive(Debug, Copy, Clone)]
pub struct LongDateTime(u64);

// difference between TTF epoch (1904-01-01 00:00:00.00) and unix epoch (1970-01-01 00:00:00.00)
const EPOCH_DIFF: i64 = 2082844800;

impl LongDateTime {
    pub fn new() -> Self {
        Self(0u64)
    }

    pub fn as_datetime(&self) -> DateTime<Utc> {
        Utc.timestamp((self.0 as i64) - EPOCH_DIFF, 0)
    }
}

impl From<u64> for LongDateTime {
    fn from(x: u64) -> Self {
        Self(x)
    }
}

impl From<DateTime<Utc>> for LongDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        Self((dt.timestamp() + EPOCH_DIFF) as u64)
    }
}

impl PackedSize for LongDateTime {
    const PACKED_LEN: usize = 8;
}

impl EncodeBE for LongDateTime {
    #[inline]
    fn encode_as_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&(self.0.to_be_bytes()));
    }
}

impl DecodeBE for LongDateTime {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Self(u64::from_be_bytes(arr))
    }
}
