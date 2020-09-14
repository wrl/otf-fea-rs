use endian_codec::{EncodeBE, DecodeBE, PackedSize};

use crate::compile_model::util::encode::*;
use crate::compile_model::util::decode::*;
use crate::parse_model as pm;

#[derive(Debug, Clone)]
pub enum Anchor {
    Coord {
        x: i16,
        y: i16
    },

    ContourCoord {
        x: i16,
        y: i16,
        contour_point: u16
    }

    // TODO: DeviceAdjustedCoord
}

impl From<&pm::Anchor> for Anchor {
    fn from(parsed: &pm::Anchor) -> Self {
        use pm::Anchor::*;

        match parsed {
            Coord { x, y } =>
                Self::Coord {
                    x: x.0 as i16,
                    y: y.0 as i16
                },

            ContourCoord { x, y, contour_point } =>
                Self::ContourCoord {
                    x: x.0 as i16,
                    y: y.0 as i16,
                    contour_point: contour_point.0 as u16
                },

            // FIXME: is "Null means 0,0" valid?
            Null =>
                Self::Coord {
                    x: 0,
                    y: 0
                },

            a => panic!("anchor try_from unimplemented for {:?}", a)
        }
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
            y: encoded.y
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
            contour_point: encoded.contour_point
        }
    }
}

impl TTFEncode for Anchor {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            &Self::Coord { x, y } =>
                buf.append(&AnchorFormat1 {
                    format: 1,
                    x,
                    y
                }),

            &Self::ContourCoord { x, y, contour_point } =>
                buf.append(&AnchorFormat2 {
                    format: 1,
                    x,
                    y,
                    contour_point
                }),
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