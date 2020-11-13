use std::convert::TryFrom;

use endian_codec::{EncodeBE, DecodeBE, PackedSize};

use crate::compile_model::util::encode::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::device::*;
use crate::compile_model::error::*;
use crate::parse_model as pm;

use crate::MaybePositioned;
use crate::compile_model::CompiledEntry;


#[derive(Debug, Clone)]
pub enum Anchor {
    Coord {
        x: MaybePositioned<i16>,
        y: MaybePositioned<i16>
    },

    ContourCoord {
        x: MaybePositioned<i16>,
        y: MaybePositioned<i16>,
        contour_point: u16
    },

    DeviceAdjustedCoord {
        x: MaybePositioned<i16>,
        y: MaybePositioned<i16>,

        x_device: Option<Device>,
        y_device: Option<Device>,
    }
}

impl Anchor {
    #[inline]
    pub fn should_encode(&self, buf: &EncodeBuf) -> bool {
        use Anchor::*;

        let optimize = buf.should_optimize_filesize();

        match self {
            Coord { x, y } => {
                (x.value != 0 || (!optimize && x.has_position()))
                    || (y.value != 0 || (!optimize && y.has_position()))
            },

            _ => true
        }
    }
}

#[inline]
fn metric_to_i16(metric: &pm::Metric) -> MaybePositioned<i16> {
    MaybePositioned {
        value: metric.value as i16,
        span: Some(metric.span.clone())
    }
}

impl TryFrom<&pm::Anchor> for Anchor {
    type Error = CompileError;

    fn try_from(parsed: &pm::Anchor) -> Result<Self, Self::Error> {
        use pm::Anchor::*;

        Ok(match parsed {
            Coord { x, y } =>
                Self::Coord {
                    x: metric_to_i16(x),
                    y: metric_to_i16(y),
                },

            ContourCoord { x, y, contour_point } =>
                Self::ContourCoord {
                    x: metric_to_i16(x),
                    y: metric_to_i16(y),
                    contour_point: contour_point.0 as u16
                },

            // FIXME: is "Null means 0,0" valid?
            Null =>
                Self::Coord {
                    x: 0.into(),
                    y: 0.into()
                },

            // FIXME: propagate device information
            DeviceAdjustedCoord { x, y } =>
                Self::DeviceAdjustedCoord {
                    x: metric_to_i16(&x.metric),
                    y: metric_to_i16(&y.metric),

                    x_device: Some(Device::try_from(&x.device)?),
                    y_device: Some(Device::try_from(&y.device)?)
                },

            Named(_) =>
                return Err(CompileError::InvalidAnchor("Named")),
        })
    }
}

#[derive(EncodeBE, DecodeBE, PackedSize)]
struct AnchorFormat1 {
    format: u16,
    x: i16,
    y: i16
}

impl From<AnchorFormat1> for Anchor {
    fn from(encoded: AnchorFormat1) -> Self {
        Self::Coord {
            x: encoded.x.into(),
            y: encoded.y.into()
        }
    }
}

#[derive(EncodeBE, DecodeBE, PackedSize)]
struct AnchorFormat2 {
    format: u16,
    x: i16,
    y: i16,
    contour_point: u16
}

impl From<AnchorFormat2> for Anchor {
    fn from(encoded: AnchorFormat2) -> Self {
        Self::ContourCoord {
            x: encoded.x.into(),
            y: encoded.y.into(),
            contour_point: encoded.contour_point
        }
    }
}

#[derive(EncodeBE, DecodeBE, PackedSize)]
struct AnchorFormat3 {
    format: u16,
    x: i16,
    y: i16,
    x_device_offset: u16,
    y_device_offset: u16
}

impl TTFEncode for Anchor {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            Self::Coord { x, y } => {
                let start = buf.append(&AnchorFormat1 {
                    format: 1,
                    x: x.value,
                    y: y.value
                })?;

                if let Some(span) = x.span.as_ref() {
                    let loc = start + u16::PACKED_LEN; // skip `format`
                    buf.add_source_map_entry(span, CompiledEntry::I16(loc));
                }

                if let Some(span) = y.span.as_ref() {
                    let loc = start + (u16::PACKED_LEN * 2); // skip `format` and `x`
                    buf.add_source_map_entry(span, CompiledEntry::I16(loc));
                }

                Ok(start)
            },

            Self::ContourCoord { x, y, contour_point } => {
                let start = buf.append(&AnchorFormat2 {
                    format: 2,
                    x: x.value,
                    y: y.value,
                    contour_point: *contour_point
                })?;

                if let Some(span) = x.span.as_ref() {
                    let loc = start + u16::PACKED_LEN; // skip `format`
                    buf.add_source_map_entry(span, CompiledEntry::I16(loc));
                }

                if let Some(span) = y.span.as_ref() {
                    let loc = start + (u16::PACKED_LEN * 2); // skip `format` and `x`
                    buf.add_source_map_entry(span, CompiledEntry::I16(loc));
                }

                Ok(start)
            },

            Self::DeviceAdjustedCoord { x, y, x_device, y_device } => {
                let start = buf.bytes.len();

                buf.defer_header_encode(
                    |buf| Ok(AnchorFormat3 {
                        format: 3,
                        x: x.value,
                        y: y.value,
                        x_device_offset:
                            match x_device {
                                Some(dev) if !dev.is_empty() => (buf.append(dev)? - start) as u16,
                                _ => 0
                            },

                        y_device_offset:
                            match y_device {
                                Some(dev) if !dev.is_empty() => (buf.append(dev)? - start) as u16,
                                _ => 0
                            }
                    }),

                    |_| Ok(()))?;

                if let Some(span) = x.span.as_ref() {
                    let loc = start + u16::PACKED_LEN; // skip `format`
                    buf.add_source_map_entry(span, CompiledEntry::I16(loc));
                }

                if let Some(span) = y.span.as_ref() {
                    let loc = start + (u16::PACKED_LEN * 2); // skip `format` and `x`
                    buf.add_source_map_entry(span, CompiledEntry::I16(loc));
                }

                Ok(start)
            }
        }
    }
}

impl TTFDecode for Anchor {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let format = decode_u16_be(bytes, 0);

        Ok(match format {
            1 => decode_from_slice::<AnchorFormat1>(bytes).into(),
            2 => decode_from_slice::<AnchorFormat2>(bytes).into(),
            _ => return Err(DecodeError::InvalidValue("format", "Anchor".into()))
        })
    }
}
