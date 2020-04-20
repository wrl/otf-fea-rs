use combine::{
    Parser,
    Stream,
    error::ParseError,
};

use crate::parser::FeaRsStream;

use super::util::*;
use super::tag::*;
use super::block::*;

#[derive(Debug)]
pub struct FeatureDefinition {
    pub tag: Tag,
    pub statements: Vec<BlockStatement>
}

pub(crate) fn feature_definition<Input>() -> impl Parser<FeaRsStream<Input>, Output = FeatureDefinition>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("feature")
        .skip(required_whitespace())

        .with(block(tag))

        .map(|block|
            FeatureDefinition {
                tag: block.ident,
                statements: block.statements
            })
}

#[derive(Debug)]
pub struct FeatureReference(pub Tag);

pub(crate) fn feature_reference<Input>() -> impl Parser<FeaRsStream<Input>, Output = FeatureReference>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("feature")
        .skip(required_whitespace())
        .with(tag())

        .map(|tag| FeatureReference(tag))
}
