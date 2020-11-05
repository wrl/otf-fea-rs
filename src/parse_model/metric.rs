use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::*;
use super::device::*;
use super::util::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceAdjustedMetric {
    pub metric: Metric,
    pub device: Device
}

impl DeviceAdjustedMetric {
    pub fn new(metric: Metric, device: Device) -> Self {
        DeviceAdjustedMetric {
            metric,
            device
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metric(pub f64);

#[inline]
pub(crate) fn metric<Input>() -> impl Parser<FeaRsStream<Input>, Output = Metric>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    decimal_number()
        .map(|x| Metric(x))
}
