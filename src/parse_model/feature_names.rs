use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    value
};

use crate::parser::FeaRsStream;

use super::block::*;
use super::name::*;
use super::util::*;

#[derive(Debug)]
pub struct FeatureNames {
    pub names: Vec<Name>
}

pub(crate) fn name_statement<Input, Ident>(_: &Ident) -> impl Parser<FeaRsStream<Input>, Output = Name>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("name")
        .skip(required_whitespace())
        .with(name())
}

#[derive(Clone, PartialEq)]
pub struct NoIdent;

impl fmt::Display for NoIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

fn no_ident<Input>() -> impl Parser<FeaRsStream<Input>, Output = NoIdent>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    value(NoIdent)
}

pub(crate) fn feature_names<Input>() -> impl Parser<FeaRsStream<Input>, Output = FeatureNames>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("featureNames")
        .skip(required_whitespace())
        .with(block(no_ident, name_statement))
        .map(|block| FeatureNames {
            names: block.statements
        })
}
