use std::fmt;
use crate::compile_model::util::decode::*;

pub struct ValueRecord {
    pub x_placement: i16,
    pub y_placement: i16,
    pub x_advance: i16,
    pub y_advance: i16,

    pub x_placement_device_offset: u16,
    pub y_placement_device_offset: u16,
    pub x_advance_device_offset: u16,
    pub y_advance_device_offset: u16
}

impl fmt::Debug for ValueRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ValueRecord");

        macro_rules! debug_if_set {
            ($field:ident) => {
                if self.$field != 0 {
                    s.field(stringify!($field), &self.$field);
                }
            }
        };

        debug_if_set!(x_placement);
        debug_if_set!(y_placement);
        debug_if_set!(x_advance);
        debug_if_set!(y_advance);

        debug_if_set!(x_placement_device_offset);
        debug_if_set!(y_placement_device_offset);
        debug_if_set!(x_advance_device_offset);
        debug_if_set!(y_advance_device_offset);

        s.finish()
    }
}

impl ValueRecord {
    pub fn decode_from_be_bytes(bytes: &[u8], mut format: u16) -> Self {
        let mut buf = [0u16; 8];
        let mut buf_idx = 0;
        let mut bytes_idx = 0;

        format &= 0x00FF;

        while format > 0 {
            let first_set = format.trailing_zeros() + 1;
            buf_idx += first_set as usize;
            format >>= first_set;

            buf[buf_idx - 1] = decode_u16_be(bytes, bytes_idx);
            bytes_idx += 2;
        }

        Self {
            x_placement: buf[0] as i16,
            y_placement: buf[1] as i16,
            x_advance: buf[2] as i16,
            y_advance: buf[3] as i16,

            x_placement_device_offset: buf[4],
            y_placement_device_offset: buf[5],
            x_advance_device_offset: buf[6],
            y_advance_device_offset: buf[7]
        }
    }
}
