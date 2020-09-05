use std::convert::TryFrom;

use crate::compile_model::error::*;

use crate::parse_model as pm;


#[derive(Debug)]
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

impl TryFrom<&pm::Anchor> for Anchor {
    type Error = CompileError;

    fn try_from(parsed: &pm::Anchor) -> CompileResult<Self> {
        use pm::Anchor::*;

        Ok(match parsed {
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
        })
    }
}
