use std::convert::Into;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    dispatch,
    value
};

use crate::parser::FeaRsStream;

use crate::parse_model::table::*;
use crate::parse_model::util::*;

#[derive(Debug)]
pub struct VertTypoAscender(isize);

#[derive(Debug)]
pub struct VertTypoDescender(isize);

#[derive(Debug)]
pub struct VertTypoLineGap(isize);

pub(crate) fn vhea_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(keyword())
        .skip(required_whitespace())
        .then(|(position, kwd)| {
            dispatch!(&*kwd;
                "VertTypoAscender" => number().map(|n| VertTypoAscender(n).into()),
                "VertTypoDescender" => number().map(|n| VertTypoDescender(n).into()),
                "VertTypoLineGap" => number().map(|n| VertTypoLineGap(n).into()),

                _ => value(position)
                .flat_map(|position|
                    crate::parse_bail!(Input, position,
                        "unexpected keyword"))
            )
        })
}
