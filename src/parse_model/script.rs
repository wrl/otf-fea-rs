use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::FeaRsStream;
use crate::Tag;

use super::util::*;
use super::tag::*;

#[derive(Debug)]
pub struct Script(pub Tag);

pub(crate) fn script<Input>() -> impl Parser<FeaRsStream<Input>, Output = Script>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("script")
        .skip(required_whitespace())
        .with(tag())
        .map(Script)
}
