use combine::{
    Parser,
    Stream,
    error::ParseError,

    look_ahead,
    attempt,
    choice,

    token,
    value
};

use crate::parser::*;
use crate::LanguageTag;

use super::util::*;
use super::tag::*;

#[derive(Debug)]
pub struct Language {
    pub tag: LanguageTag,
    pub include_default: bool,
    pub required: bool
}

pub(crate) fn language_tag<Input>() -> impl Parser<FeaRsStream<Input>, Output = LanguageTag>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    tag_storage()
        .map(LanguageTag)
}

pub(crate) fn language<Input>() -> impl Parser<FeaRsStream<Input>, Output = Language>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("language")
        .skip(required_whitespace())
        .with(language_tag())

        .and(choice((
            look_ahead(token(b';')).map(|_| None),
            attempt(required_whitespace()
                .with(choice((
                    literal_ignore_case("include_dflt").map(|_| Some(true)),
                    literal_ignore_case("exclude_dflt").map(|_| Some(false)),

                    // FIXME: these are deprecated and should display a warning
                    literal_ignore_case("includeDFLT").map(|_| Some(true)),
                    literal_ignore_case("excludeDFLT").map(|_| Some(false)),

                    value(None)
                ))))
        )))

        .and(choice((
            look_ahead(token(b';')).map(|_| false),
            attempt(required_whitespace()
                .with(literal_ignore_case("required").map(|_| true)))
        )))

        .map(|((tag, include_default), required)| {
            Language {
                tag,
                include_default: include_default.unwrap_or(true),
                required
            }
        })
}
