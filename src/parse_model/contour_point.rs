use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::*;
use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContourPoint(pub usize);

#[inline]
pub(crate) fn contour_point<Input>() -> impl Parser<FeaRsStream<Input>, Output = ContourPoint>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("contourpoint")
        .skip(required_whitespace())
        .with(number())
        .map(|x| ContourPoint(x as usize))
}
