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
    // to keep encoded data size as small as possible, ValueRecords can be encoded to just a subset
    // of their fields - down to and including 0 fields in some cases. the presence of each field
    // in the encoded representation is indicated by a set bit flag in the `format` variable.

    #[allow(unused_assignments)]
    pub fn decode_from_be_bytes(bytes: &[u8], format: u16) -> Self {
        let mut ret = Self {
            x_placement: 0,
            y_placement: 0,
            x_advance: 0,
            y_advance: 0,

            x_placement_device_offset: 0,
            y_placement_device_offset: 0,
            x_advance_device_offset: 0,
            y_advance_device_offset: 0
        };

        let mut bytes_idx = 0;

        macro_rules! read_if_in_format {
            ($shift:expr, $var:ident, $t:ty) => {
                if (format & (1u16 << $shift)) != 0 {
                    ret.$var = decode_u16_be(bytes, bytes_idx) as $t;
                    bytes_idx += 2;
                }
            }
        }

        read_if_in_format!(0, x_placement, i16);
        read_if_in_format!(1, y_placement, i16);
        read_if_in_format!(2, x_advance, i16);
        read_if_in_format!(3, y_advance, i16);

        read_if_in_format!(4, x_placement_device_offset, u16);
        read_if_in_format!(5, y_placement_device_offset, u16);
        read_if_in_format!(6, x_advance_device_offset, u16);
        read_if_in_format!(7, y_advance_device_offset, u16);

        ret
    }

    #[allow(unused_assignments)]
    pub fn encode_to_be_bytes(&self, bytes: &mut [u8], format: u16) {
        let mut bytes_idx = 0;

        macro_rules! write_if_in_format {
            ($shift:expr, $var:ident) => {
                if (format & (1u16 << $shift)) != 0 {
                    &bytes[bytes_idx..bytes_idx + 2]
                        .copy_from_slice(&self.$var.to_be_bytes());
                    bytes_idx += 2;
                }
            }
        }

        write_if_in_format!(0, x_placement);
        write_if_in_format!(1, y_placement);
        write_if_in_format!(2, x_advance);
        write_if_in_format!(3, y_advance);

        write_if_in_format!(4, x_placement_device_offset);
        write_if_in_format!(5, y_placement_device_offset);
        write_if_in_format!(6, x_advance_device_offset);
        write_if_in_format!(7, y_advance_device_offset);
    }
}