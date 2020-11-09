mod checksum;
pub use checksum::*;

mod checked_from;
pub use checked_from::*;

pub mod decode;
pub mod encode;

mod fixed1616;
pub use fixed1616::Fixed1616;

mod long_date_time;
pub use long_date_time::LongDateTime;

#[macro_use]
mod tag;

mod ttf_version;
pub use ttf_version::TTFVersion;

mod ttf_tagged;
pub use ttf_tagged::TTFTagged;

pub(crate) const fn align_len(len: usize) -> usize {
    let round_up = (4usize - (len & 0x3)) & 0x3;
    return len + round_up;
}
