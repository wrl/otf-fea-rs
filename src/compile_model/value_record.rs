use std::convert::TryFrom;
use std::fmt;

use endian_codec::PackedSize;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::device::*;
use crate::compile_model::error::*;

use crate::parse_model as pm;

use crate::MaybePositioned;
use crate::compile_model::CompiledEntry;


#[derive(PartialEq, Eq, Clone)]
pub struct ValueRecord {
    pub x_placement: MaybePositioned<i16>,
    pub y_placement: MaybePositioned<i16>,
    pub x_advance: MaybePositioned<i16>,
    pub y_advance: MaybePositioned<i16>,

    pub x_placement_device: Option<Device>,
    pub y_placement_device: Option<Device>,
    pub x_advance_device: Option<Device>,
    pub y_advance_device: Option<Device>
}

impl fmt::Debug for ValueRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ValueRecord");

        macro_rules! debug_if_set {
            ($field:ident) => {
                if self.$field.value != 0 {
                    s.field(stringify!($field), &self.$field);
                }
            }
        };

        macro_rules! debug_if_some {
            ($field:ident) => {
                if let Some(ref field) = self.$field {
                    s.field(stringify!($field), field);
                }
            }
        };

        debug_if_set!(x_placement);
        debug_if_set!(y_placement);
        debug_if_set!(x_advance);
        debug_if_set!(y_advance);

        debug_if_some!(x_placement_device);
        debug_if_some!(y_placement_device);
        debug_if_some!(x_advance_device);
        debug_if_some!(y_advance_device);

        s.finish()
    }
}

impl ValueRecord {
    #[inline]
    pub fn zero() -> Self {
        Self {
            x_placement: 0.into(),
            y_placement: 0.into(),
            x_advance: 0.into(),
            y_advance: 0.into(),

            x_placement_device: None,
            y_placement_device: None,
            x_advance_device: None,
            y_advance_device: None
        }
    }

    // to keep encoded data size as small as possible, ValueRecords can be encoded to just a subset
    // of their fields - down to and including 0 fields in some cases. the presence of each field
    // in the encoded representation is indicated by a set bit flag in the `format` variable.

    #[allow(unused_assignments)]
    pub fn decode_from_format(bytes: &[u8], format: u16) -> Self {
        let mut ret = Self {
            x_placement: 0.into(),
            y_placement: 0.into(),
            x_advance: 0.into(),
            y_advance: 0.into(),

            x_placement_device: None,
            y_placement_device: None,
            x_advance_device: None,
            y_advance_device: None
        };

        let mut bytes_idx = 0;

        macro_rules! read_if_in_format {
            ($shift:expr, $var:ident, $t:ty) => {
                if (format & (1u16 << $shift)) != 0 {
                    ret.$var.value = decode_u16_be(bytes, bytes_idx) as $t;
                    bytes_idx += 2;
                }
            }
        }

        read_if_in_format!(0, x_placement, i16);
        read_if_in_format!(1, y_placement, i16);
        read_if_in_format!(2, x_advance, i16);
        read_if_in_format!(3, y_advance, i16);

        // FIXME: decode device
        // read_if_in_format!(4, x_placement_device_offset, u16);
        // read_if_in_format!(5, y_placement_device_offset, u16);
        // read_if_in_format!(6, x_advance_device_offset, u16);
        // read_if_in_format!(7, y_advance_device_offset, u16);

        ret
    }

    #[inline]
    pub fn size_for_format(format: u16) -> usize {
        (format.count_ones() as usize) * u16::PACKED_LEN
    }

    #[inline]
    pub fn smallest_possible_format(&self) -> u16 {
        let mut ret = 0u16;

        macro_rules! set_bit_if_nonzero {
            ($shift:expr, $var:ident) => {
                ret |= ((self.$var.value != 0) as u16) << $shift;
            }
        }

        macro_rules! set_bit_if_some {
            ($shift:expr, $var:ident) => {
                ret |= ((self.$var.is_some()) as u16) << $shift;
            }
        }

        set_bit_if_nonzero!(0, x_placement);
        set_bit_if_nonzero!(1, y_placement);
        set_bit_if_nonzero!(2, x_advance);
        set_bit_if_nonzero!(3, y_advance);

        set_bit_if_some!(4, x_placement_device);
        set_bit_if_some!(5, y_placement_device);
        set_bit_if_some!(6, x_advance_device);
        set_bit_if_some!(7, y_advance_device);

        ret
    }

    #[allow(unused_assignments)]
    pub fn encode_to_format(&self, buf: &mut EncodeBuf, format: u16, parent_table_start: usize, mut start: usize)
            -> EncodeResult<()> {
        macro_rules! write_if_in_format {
            ($shift:expr, $var:ident) => {
                if (format & (1u16 << $shift)) != 0 {
                    let loc = buf.encode_at(&self.$var.value, start)?;
                    start += i16::PACKED_LEN;

                    if let Some(span) = &self.$var.span {
                        buf.add_source_map_entry(span, CompiledEntry::I16(loc));
                    }
                }
            }
        }

        write_if_in_format!(0, x_placement);
        write_if_in_format!(1, y_placement);
        write_if_in_format!(2, x_advance);
        write_if_in_format!(3, y_advance);

        macro_rules! write_if_device_in_format {
            ($shift:expr, $var:ident) => {
                if (format & (1u16 << $shift)) != 0 {
                    let offset = match self.$var.as_ref() {
                        Some(dev) if !dev.is_empty() =>
                            (buf.append(dev)? - parent_table_start) as u16,

                        _ => 0u16
                    };

                    buf.encode_at(&offset, start)?;
                    start += u16::PACKED_LEN;
                }
            }
        }

        write_if_device_in_format!(4, x_placement_device);
        write_if_device_in_format!(5, y_placement_device);
        write_if_device_in_format!(6, x_advance_device);
        write_if_device_in_format!(7, y_advance_device);

        Ok(())
    }
}


pub trait ValueRecordFromParsed<T>: Sized {
    fn from_parsed(parsed: T, vertical: bool) -> CompileResult<Self>;
}

#[inline]
fn metric_to_i16_checked(m: &pm::Metric) -> CompileResult<MaybePositioned<i16>> {
    // FIXME: actually check

    Ok(MaybePositioned {
        value: m.value.trunc() as i16,
        span: Some(m.span.clone())
    })
}

impl ValueRecordFromParsed<&pm::ValueRecord> for ValueRecord {
    // FIXME: return a Result<> if the f64 -> i16 fails
    fn from_parsed(parsed: &pm::ValueRecord, vertical: bool) -> CompileResult<Self> {
        use pm::ValueRecord::*;

        Ok(match parsed {
            Advance(metric) if vertical => Self {
                y_advance: metric_to_i16_checked(metric)?,
                ..Self::zero()
            },

            Advance(metric) => Self {
                x_advance: metric_to_i16_checked(metric)?,
                ..Self::zero()
            },

            PlacementAdvance {
                x_placement, y_placement,
                x_advance, y_advance
            } => Self {
                x_placement: metric_to_i16_checked(x_placement)?,
                y_placement: metric_to_i16_checked(y_placement)?,
                x_advance: metric_to_i16_checked(x_advance)?,
                y_advance: metric_to_i16_checked(y_advance)?,

                ..Self::zero()
            },

            // FIXME: device
            DeviceAdjusted {
                x_placement, y_placement, x_advance, y_advance
            } => Self {
                x_placement: metric_to_i16_checked(&x_placement.metric)?,
                y_placement: metric_to_i16_checked(&y_placement.metric)?,
                x_advance: metric_to_i16_checked(&x_advance.metric)?,
                y_advance: metric_to_i16_checked(&y_advance.metric)?,

                x_placement_device: Some(Device::try_from(&x_placement.device)?),
                y_placement_device: Some(Device::try_from(&y_placement.device)?),
                x_advance_device: Some(Device::try_from(&x_advance.device)?),
                y_advance_device: Some(Device::try_from(&y_advance.device)?),

                ..Self::zero()
            },

            Null => Self::zero(),
        })
    }
}

impl ValueRecordFromParsed<&Option<pm::ValueRecord>> for ValueRecord {
    fn from_parsed(parsed: &Option<pm::ValueRecord>, vertical: bool) -> CompileResult<Self> {
        parsed.as_ref()
            .map(|vr| Self::from_parsed(vr, vertical))
            .unwrap_or_else(|| Ok(Self::zero()))
    }
}
