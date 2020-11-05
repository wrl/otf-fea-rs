use combine::{
    Parser,
    Stream,
    error::ParseError,

    optional,

    parser::byte::space,
};

use crate::parser::*;
use super::util::*;

#[derive(Debug)]
pub struct Parameters {
    pub design_size: f64,
    pub subfamily_id: isize,

    pub range_start: Option<f64>,
    pub range_end: Option<f64>
}

pub(crate) fn parameters<Input>() -> impl Parser<FeaRsStream<Input>, Output = Parameters>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("parameters")
        .skip(required_whitespace())
        .with(decimal_number())
        .skip(required_whitespace())
        .and(number())

        // FIXME: also need to parse these if subfamily ID is nonzero
        .and(optional(space()
                .skip(optional_whitespace())
                .with(decimal_number())
                .skip(required_whitespace())
                .and(decimal_number())))

        .map(|((design_size, subfamily_id), range)| {
            let (range_start, range_end) = match range {
                Some((s, e)) => (Some(s), Some(e)),
                None => (None, None)
            };

            Parameters {
                design_size,
                subfamily_id,

                range_start,
                range_end
            }
        })
}
