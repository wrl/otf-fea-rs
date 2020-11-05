use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::*;
use super::device::*;

use super::util::*;

use crate::Positioned;
use super::positioned::*;


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

pub type Metric = Positioned<f64>;

#[inline]
pub(crate) fn metric<Input>() -> impl Parser<FeaRsStream<Input>, Output = Metric>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    positioned(decimal_number())
}
