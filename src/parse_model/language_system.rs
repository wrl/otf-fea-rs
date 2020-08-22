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
pub struct LanguageSystem {
    pub script: Tag,
    pub language: Tag
}

pub(crate) fn language_system<Input>() -> impl Parser<FeaRsStream<Input>, Output = LanguageSystem>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("languagesystem")
        .skip(required_whitespace())
        .with(tag())
        .skip(required_whitespace())
        .and(tag())

        .map(|(script, language)| LanguageSystem {
            script,
            language
        })
}
