use std::convert::TryFrom;

use endian_codec::{EncodeBE, DecodeBE, PackedSize};

use crate::compile_model::util::encode::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::error::*;
use crate::parse_model as pm;

use crate::SourceSpan;
use crate::compile_model::CompiledEntry;



#[derive(Debug, Clone)]
pub enum Anchor {
    Coord {
        x: i16,
        y: i16,

        x_span: Option<SourceSpan>,
        y_span: Option<SourceSpan>
    },

    ContourCoord {
        x: i16,
        y: i16,
        contour_point: u16,

        x_span: Option<SourceSpan>,
        y_span: Option<SourceSpan>
    },

    DeviceAdjustedCoord {
        x: i16,
        y: i16
    }
}

impl TryFrom<&pm::Anchor> for Anchor {
    type Error = CompileError;

    fn try_from(parsed: &pm::Anchor) -> Result<Self, Self::Error> {
        use pm::Anchor::*;

        Ok(match parsed {
            Coord { x, y } =>
                Self::Coord {
                    x: x.value as i16,
                    y: y.value as i16,

                    x_span: Some(x.span.clone()),
                    y_span: Some(y.span.clone())
                },

            ContourCoord { x, y, contour_point } =>
                Self::ContourCoord {
                    x: x.value as i16,
                    y: y.value as i16,
                    contour_point: contour_point.0 as u16,

                    x_span: Some(x.span.clone()),
                    y_span: Some(y.span.clone())
                },

            // FIXME: is "Null means 0,0" valid?
            Null =>
                Self::Coord {
                    x: 0,
                    y: 0,

                    x_span: None,
                    y_span: None,
                },

            // FIXME: propagate device information
            DeviceAdjustedCoord { x, y } =>
                Self::Coord {
                    x: x.metric.value as i16,
                    y: y.metric.value as i16,

                    x_span: None,
                    y_span: None,
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
            x: encoded.x,
            y: encoded.y,

            x_span: None,
            y_span: None
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
            x: encoded.x,
            y: encoded.y,
            contour_point: encoded.contour_point,

            x_span: None,
            y_span: None
        }
    }
}

impl TTFEncode for Anchor {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            &Self::Coord { x, y, ref x_span, ref y_span } => {
                let start = buf.append(&AnchorFormat1 {
                    format: 1,
                    x,
                    y
                })?;

                if let Some(x_span) = x_span {
                    let x_loc = start + u16::PACKED_LEN; // skip `format`
                    buf.add_source_map_entry(x_span, CompiledEntry::I16(x_loc));
                }

                if let Some(y_span) = y_span {
                    let y_loc = start + (u16::PACKED_LEN * 2); // skip `format` and `x`
                    buf.add_source_map_entry(y_span, CompiledEntry::I16(y_loc));
                }

                Ok(start)
            },

            &Self::ContourCoord { x, y, contour_point , ref x_span, ref y_span } => {
                let start = buf.append(&AnchorFormat2 {
                    format: 1,
                    x,
                    y,
                    contour_point
                })?;

                if let Some(x_span) = x_span {
                    let x_loc = start + u16::PACKED_LEN; // skip `format`
                    buf.add_source_map_entry(x_span, CompiledEntry::I16(x_loc));
                }

                if let Some(y_span) = y_span {
                    let y_loc = start + (u16::PACKED_LEN * 2); // skip `format` and `x`
                    buf.add_source_map_entry(y_span, CompiledEntry::I16(y_loc));
                }

                Ok(start)
            },

            &Self::DeviceAdjustedCoord { .. } =>
                panic!("unimplemented device encode")
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
