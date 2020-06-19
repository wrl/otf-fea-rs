mod checksum;
pub use checksum::*;

mod fixed1616;
pub use fixed1616::Fixed1616;

mod long_date_time;
pub use long_date_time::LongDateTime;

#[macro_use]
mod tag;

mod ttf_version;
pub use ttf_version::TTFVersion;

#[inline]
pub(crate) fn decode_u16_be(bytes: &[u8], offset: usize) -> u16 {
    let mut a = [0u8; 2];
    a.copy_from_slice(&bytes[offset..offset+2]);
    u16::from_be_bytes(a)
}
