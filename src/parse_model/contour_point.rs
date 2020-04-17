use std::cmp;

use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::FeaRsStream;
use super::util::*;

#[derive(Debug, cmp::PartialEq)]
pub struct ContourPoint(pub isize);

#[inline]
pub(crate) fn contour_point<Input>() -> impl Parser<FeaRsStream<Input>, Output = ContourPoint>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("contourpoint")
        .skip(required_whitespace())
        .with(number())
        .map(|x| ContourPoint(x as isize))
}
