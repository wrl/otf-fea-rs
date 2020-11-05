use combine::{
    Parser,
    Stream,
    error::ParseError,

    dispatch,
    value
};

use crate::parser::*;

use crate::parse_model::table::*;
use crate::parse_model::metric::*;
use crate::parse_model::util::*;

#[derive(Debug)]
pub struct CaretOffset(Metric);

#[derive(Debug)]
pub struct Ascender(Metric);

#[derive(Debug)]
pub struct Descender(Metric);

#[derive(Debug)]
pub struct LineGap(Metric);

pub(crate) fn hhea_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(keyword())
        .skip(required_whitespace())
        .then(|(position, kwd)| {
            dispatch!(&*kwd;
                "CaretOffset" => metric().map(|m| CaretOffset(m).into()),
                "Ascender" => metric().map(|m| Ascender(m).into()),
                "Descender" => metric().map(|m| Descender(m).into()),
                "LineGap" => metric().map(|m| LineGap(m).into()),

                _ => value(position)
                .flat_map(|position|
                    crate::parse_bail!(Input, position,
                        "unexpected keyword"))
            )
        })
}
