use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::*;
use crate::{
    ScriptTag,
    LanguageTag
};

use super::language::*;
use super::script::*;
use super::util::*;

#[derive(Debug)]
pub struct LanguageSystem {
    pub script: ScriptTag,
    pub language: LanguageTag
}

pub(crate) fn language_system<Input>() -> impl Parser<FeaRsStream<Input>, Output = LanguageSystem>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("languagesystem")
        .skip(required_whitespace())
        .with(script_tag())
        .skip(required_whitespace())
        .and(language_tag())

        .map(|(script, language)| LanguageSystem {
            script,
            language
        })
}
