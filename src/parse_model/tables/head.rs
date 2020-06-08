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
pub struct FontRevision(pub f64);

pub(crate) fn head_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(keyword())
        .skip(required_whitespace())
        .then(|(position, kwd)| {
            dispatch!(&*kwd;
                "FontRevision" => decimal_number()
                    .map(|fr| FontRevision(fr).into()),

                _ => value(position)
                .flat_map(|position|
                    crate::parse_bail!(Input, position,
                        "unexpected keyword"))
            )
        })
}
