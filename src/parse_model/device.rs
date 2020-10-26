use combine::{
    Parser,
    Stream,
    error::ParseError,

    token,

    parser::repeat::sep_by,

    choice
};

use crate::parser::FeaRsStream;
use super::util::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceAdjustment {
    pub ppem_size: isize,
    pub pixel_adjustment: isize
}

#[derive(Debug, Clone, PartialEq)]
pub enum Device {
    Adjustments(Vec<DeviceAdjustment>),
    Null
}

#[inline]
pub(crate) fn device<Input>() -> impl Parser<FeaRsStream<Input>, Output = Device>
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

    literal_ignore_case("<device")
        .skip(required_whitespace())
        .with(choice((
            literal("NULL")
                .skip(close())
                .map(|_| Device::Null),

            sep_by(optional_whitespace()
                .with(number())
                    .skip(required_whitespace())
                    .and(number())
                    .skip(optional_whitespace())
                    .map(|(ppem_size, pixel_adjustment)| {
                        DeviceAdjustment {
                            ppem_size,
                            pixel_adjustment
                        }
                    }), token(b','))
                .skip(close())
                .map(|a| Device::Adjustments(a))
        )))
}
