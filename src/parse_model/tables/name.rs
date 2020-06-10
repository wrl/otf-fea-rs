use combine::{
    Parser,
    Stream,
    error::ParseError,

    dispatch,
    optional,
    value
};

use crate::parser::FeaRsStream;

use crate::parse_model::table::*;
use crate::parse_model::util::*;
use crate::parse_model::string::*;

#[derive(Debug)]
pub struct NameId {
    name_id: usize,
    platform_id: usize,
    platform_enc_id: usize,
    lang_id: usize,
    string: String
}

fn nameid<Input>() -> impl Parser<FeaRsStream<Input>, Output = NameId>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(uinteger())
        .skip(required_whitespace())
        .and(optional(
            uinteger()
                .skip(required_whitespace())
                .and(optional(
                    uinteger()
                        .skip(required_whitespace())
                        .and(uinteger())
                        .skip(required_whitespace())
                ))
        ))

        .flat_map(|((position, name_id), attributes)| {
            let attributes = match attributes {
                // mac
                Some((1, Some((enc, lang)))) => (1, enc, lang),
                Some((1, None)) => (1, 0, 0),

                // windows
                Some((3, Some((enc, lang)))) => (3, enc, lang),
                Some((3, None))
                    | None => (3, 1, 0x0409),

                Some((p, _)) => crate::parse_bail!(Input, position,
                    format!("expected platform ID 1 or 3 (got {})", p))
            };

            Ok((name_id, attributes))
        })

        .then_ref(|(_, (platform_id, ..))|
            match platform_id {
                1 => string_mac_escaped().left(),
                3 => string_win_escaped().right(),
                _ => unreachable!()
            }
        )

        .flat_map(|((name_id, attributes), string)| {
            let (platform_id, platform_enc_id, lang_id) = attributes;

            Ok(NameId {
                name_id,

                platform_id,
                platform_enc_id,
                lang_id,

                string,
            })
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
