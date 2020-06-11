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
use crate::parse_model::name::*;

#[derive(Debug)]
pub struct NameId {
    pub name_id: isize,
    pub platform_id: isize,
    pub platform_enc_id: isize,
    pub language_id: isize,
    pub name: String
}

fn nameid<Input>() -> impl Parser<FeaRsStream<Input>, Output = NameId>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    number()
        .skip(required_whitespace())
        .and(name())

        .map(|(name_id, name)| {
            NameId {
                name_id,

                platform_id: name.platform_id,
                platform_enc_id: name.script_id,
                language_id: name.language_id,

                name: name.name,
            }
        })
}

pub(crate) fn name_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(keyword())
        .skip(required_whitespace())
        .then(|(position, kwd)| {
            dispatch!(&*kwd;
                "nameid" => nameid().map(TableStatement::from),

                _ => value(position)
                .flat_map(|position|
                    crate::parse_bail!(Input, position,
                        "unexpected keyword"))
            )
        })
}
