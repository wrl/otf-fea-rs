use std::fmt;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;

use crate::parse_model as pm;

#[derive(Hash, PartialEq, Eq, Clone)]
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

// FIXME: actually check
#[inline]
fn metric_to_i16_checked(x: &pm::Metric) -> i16 {
    x.0.trunc() as i16
}

impl ValueRecord {
    #[inline]
    pub fn zero() -> Self {
        Self {
            x_placement: 0,
            y_placement: 0,
            x_advance: 0,
            y_advance: 0,

            x_placement_device_offset: 0,
            y_placement_device_offset: 0,
            x_advance_device_offset: 0,
            y_advance_device_offset: 0
        }
    }

    // to keep encoded data size as small as possible, ValueRecords can be encoded to just a subset
    // of their fields - down to and including 0 fields in some cases. the presence of each field
    // in the encoded representation is indicated by a set bit flag in the `format` variable.

    #[allow(unused_assignments)]
    pub fn decode_from_format(bytes: &[u8], format: u16) -> Self {
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

    #[inline]
    pub fn smallest_possible_format(&self) -> u16 {
        let mut ret = 0u16;

        macro_rules! set_bit_if_nonzero {
            ($shift:expr, $var:ident) => {
                ret |= ((self.$var != 0) as u16) << $shift;
            }
        }

        set_bit_if_nonzero!(0, x_placement);
        set_bit_if_nonzero!(1, y_placement);
        set_bit_if_nonzero!(2, x_advance);
        set_bit_if_nonzero!(3, y_advance);

        set_bit_if_nonzero!(4, x_placement_device_offset);
        set_bit_if_nonzero!(5, y_placement_device_offset);
        set_bit_if_nonzero!(6, x_advance_device_offset);
        set_bit_if_nonzero!(7, y_advance_device_offset);

        ret
    }

    #[allow(unused_assignments)]
    pub fn encode_to_format(&self, buf: &mut EncodeBuf, format: u16) -> EncodeResult<()> {
        macro_rules! write_if_in_format {
            ($shift:expr, $var:ident) => {
                if (format & (1u16 << $shift)) != 0 {
                    buf.append(&self.$var)?;
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

        Ok(())
    }
}

pub trait ValueRecordFromParsed<T> {
    fn from_parsed(parsed: &T, vertical: bool) -> Self;
}

impl ValueRecordFromParsed<pm::ValueRecord> for ValueRecord {
    // FIXME: return a result if the f64 -> i16 fails
    fn from_parsed(parsed: &pm::ValueRecord, vertical: bool) -> Self {
        use pm::ValueRecord::*;

        match parsed {
            Advance(metric) if vertical => Self {
                y_advance: metric_to_i16_checked(metric),
                ..Self::zero()
            },

            Advance(metric) => Self {
                x_advance: metric_to_i16_checked(metric),
                ..Self::zero()
            },

            PlacementAdvance {
                x_placement, y_placement,
                x_advance, y_advance
            } => Self {
                x_placement: metric_to_i16_checked(x_placement),
                y_placement: metric_to_i16_checked(y_placement),
                x_advance: metric_to_i16_checked(x_advance),
                y_advance: metric_to_i16_checked(y_advance),

                ..Self::zero()
            },

            DeviceAdjusted { .. } => panic!(),

            Null => Self::zero(),
        }
    }
}
