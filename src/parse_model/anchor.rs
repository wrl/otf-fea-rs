use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    token,
    choice
};

use crate::parser::FeaRsStream;
use super::contour_point::*;
use super::metric::*;
use super::device::*;
use super::glyph::*;
use super::util::*;

pub struct AnchorName(pub GlyphNameStorage);

impl fmt::Debug for AnchorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnchorName(\"")?;

        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        write!(f, "\")")
    }
}

#[derive(Debug)]
pub struct AnchorDefinition {
    pub name: AnchorName,
    pub anchor: Anchor
}

#[derive(Debug)]
pub enum Anchor {
    Coord {
        x: Metric,
        y: Metric
    },

    ContourCoord {
        x: Metric,
        y: Metric,

        contour_point: ContourPoint
    },

    DeviceAdjustedCoord {
        x: DeviceAdjustedMetric,
        y: DeviceAdjustedMetric
    },

    Named(AnchorName),

    Null
}

pub(crate) fn anchor_definition<Input>() -> impl Parser<FeaRsStream<Input>, Output = AnchorDefinition>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("anchorDef")
        .skip(required_whitespace())
        .with(metric())
        .skip(required_whitespace())
        .and(metric())
        .skip(required_whitespace())
        .and(choice!(
                contour_point()
                .skip(required_whitespace())
                .and(glyph_name_unwrapped())
                .map(|(point, name)| Either2::A((point, AnchorName(name)))),

                glyph_name_unwrapped()
                .map(|a| Either2::B(AnchorName(a)))
        ))

        .map(|((x, y), more)| {
            match more {
                Either2::A((contour_point, name)) =>
                    AnchorDefinition {
                        name,
                        anchor: Anchor::ContourCoord { x, y, contour_point }
                    },

                Either2::B(name) => 
                    AnchorDefinition {
                        name,
                        anchor: Anchor::Coord { x, y }
                    }
            }
        })
}

pub(crate) fn anchor<Input>() -> impl Parser<FeaRsStream<Input>, Output = Anchor>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    #[inline]
    pub(crate) fn close<Input>() -> impl Parser<FeaRsStream<Input>, Output = ()>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        token(b'>')
            .map(|_| ())
    }

    literal_ignore_case("<anchor")
        .skip(required_whitespace())
        .with(choice!(
            literal("NULL")
                .skip(close())
                .map(|_| Anchor::Null),

            metric()
                .skip(required_whitespace())
                .and(metric())
                .and(choice!(
                    close()
                        .map(|a| Either3::A(a)),

                    required_whitespace()
                        .with(choice!(
                            contour_point()
                                .skip(close())
                                .map(|b| Either3::B(b)),

                            device()
                                .skip(required_whitespace())
                                .and(device())
                                .skip(close())
                                .map(|devices| Either3::C(devices))))

                        // FIXME: need device table form
                    ))

                .map(|((x, y), more)| {
                    match more {
                        Either3::A(_) => 
                            Anchor::Coord { x, y },

                        Either3::B(contour_point) =>
                            Anchor::ContourCoord { x, y, contour_point },

                        Either3::C(devices) =>
                            Anchor::DeviceAdjustedCoord {
                                x: DeviceAdjustedMetric::new(x, devices.0),
                                y: DeviceAdjustedMetric::new(y, devices.1)
                            }
                    }
                }),

            glyph_name_unwrapped()
                .skip(close())
                .map(|n| Anchor::Named(AnchorName(n)))
        ))
}
