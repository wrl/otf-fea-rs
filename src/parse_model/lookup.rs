use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::FeaRsStream;
use super::util::*;
use super::lookup_definition::*;

#[derive(Debug)]
pub struct Lookup(LookupBlockLabel);

pub(crate) fn lookup<Input>() -> impl Parser<FeaRsStream<Input>, Output = Lookup>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("lookup")
        .skip(required_whitespace())
        .with(lookup_block_label())
        .map(|label| Lookup(label))
}
