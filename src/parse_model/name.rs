use combine::{
    Parser,
    Stream,
    error::ParseError,

    optional,
};

use crate::parser::FeaRsStream;

use super::string::*;
use super::util::*;

#[derive(Debug)]
pub struct Name {
    pub platform_id: isize,
    pub script_id: isize,
    pub language_id: isize,
    pub name: String
}

#[derive(Debug)]
enum Platform {
    Mac = 1,
    Windows = 3
}

pub(crate) fn name<Input>() -> impl Parser<FeaRsStream<Input>, Output = Name>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    optional(
        combine::position()
            .and(number())
            .flat_map(|(position, n)| {
                Ok(match n {
                    1 => Platform::Mac,
                    3 => Platform::Windows,
                    _ => crate::parse_bail!(Input, position,
                        "expected platform id 1 or 3")
                })
            })

            .skip(required_whitespace())
            .and(optional(
                number()
                    .skip(required_whitespace())
                    .and(number())
                    .skip(required_whitespace())))
        )

        .map(|ids| ids.unwrap_or((Platform::Windows, None)))

        .then_ref(|(platform, ..)|
            match platform {
                Platform::Mac => string_mac_escaped().left(),
                Platform::Windows => string_win_escaped().right(),
            }
        )

        .map(|((platform_id, ids), name)| {
            let (script_id, language_id) = ids.unwrap_or_else(|| {
                match platform_id {
                    Platform::Mac => (0, 0),
                    Platform::Windows => (1, 0x409),
                }
            });

            Name {
                platform_id: platform_id as isize,
                script_id,
                language_id,
                name
            }
        })
}
