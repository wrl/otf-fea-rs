use combine::{
    Parser,
    Stream,
    error::ParseError,

    parser,

    token,
    optional,

    parser::byte::newline,
    parser::repeat::skip_until,
    choice
};

use crate::parser::FeaRsStream;
use super::util::*;
use super::tag::*;

#[derive(Debug)]
pub struct Anonymous(pub Tag);

pub(crate) fn anonymous<Input>() -> impl Parser<FeaRsStream<Input>, Output = Anonymous>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .skip(literal_ignore_case("anon"))
        .skip(optional(literal_ignore_case("ymous")))
        .skip(required_whitespace())
        .and(tag())
        .skip(optional_whitespace())
        .skip(token(b'{'))
        .skip(skip_until(newline()).skip(newline()))

        .and(parser(|input| {
                let mut parse_iter = choice((
                        token(b'}')
                        .skip(optional_whitespace())
                        .with(tag())
                        .map(Some),

                        skip_until(newline()).skip(newline()).map(|_| None)
                ))
                    .iter(input);

                for line in &mut parse_iter {
                    if line.is_some() {
                        return parse_iter.into_result(line);
                    }
                }

                parse_iter.into_result(None)
        }))

        .flat_map(|((position, opening_ident), closing_ident)| {
            let closing_ident = match closing_ident {
                Some(i) => i,
                None => crate::parse_bail!(Input, position,
                    "unclosed anonymous block")
            };

            if opening_ident != closing_ident {
                crate::parse_bail!(Input, position,
                    format!("mismatched block identifier (opening \"{}\", closing\"{}\")",
                    opening_ident, closing_ident));
            }

            Ok(Anonymous(opening_ident))
        })
}
