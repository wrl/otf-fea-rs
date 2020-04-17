use combine::{
    Parser,
    Stream,
    error::ParseError,

    token,
    choice
};

use crate::parser::FeaRsStream;
use super::metric::*;
use super::device::*;
use super::util::*;

#[derive(Debug)]
pub enum ValueRecord {
    // format A
    Advance(Metric),

    // format B
    PlacementAdvance {
        x_placement: Metric,
        y_placement: Metric,
        x_advance: Metric,
        y_advance: Metric,
    },

    // format C
    // spec says unimplemented, but feaLib implements it
    DeviceAdjusted {
        x_placement: DeviceAdjustedMetric,
        y_placement: DeviceAdjustedMetric,
        x_advance: DeviceAdjustedMetric,
        y_advance: DeviceAdjustedMetric
    }
}

pub(crate) fn value_record<Input>() -> impl Parser<FeaRsStream<Input>, Output = ValueRecord>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    choice((
        token(b'<').skip(optional_whitespace())
            .with(metric()).skip(required_whitespace())
            .and(metric()).skip(required_whitespace())
            .and(metric()).skip(required_whitespace())
            .and(metric())
            .and(choice!(
                    required_whitespace()
                        .with(device())
                        .skip(required_whitespace())
                        .and(device())
                        .skip(required_whitespace())
                        .and(device())
                        .skip(required_whitespace())
                        .and(device())
                        .skip(optional_whitespace())
                        .skip(token(b'>'))
                        .map(|(((d1, d2), d3), d4)| Some((d1, d2, d3, d4))),

                    optional_whitespace()
                        .skip(token(b'>'))
                        .map(|_| None)
            ))

            .map(|((((x_placement, y_placement), x_advance), y_advance), devices)| {
                if let Some(devices) = devices {
                    ValueRecord::DeviceAdjusted {
                        x_placement: DeviceAdjustedMetric::new(x_placement, devices.0),
                        y_placement: DeviceAdjustedMetric::new(y_placement, devices.1),
                        x_advance: DeviceAdjustedMetric::new(x_advance, devices.2),
                        y_advance: DeviceAdjustedMetric::new(y_advance, devices.3)
                    }
                } else {
                    ValueRecord::PlacementAdvance {
                        x_placement,
                        y_placement,
                        x_advance,
                        y_advance,
                    }
                }
            }),

        metric()
            .map(|m| ValueRecord::Advance(m))
    ))
}
