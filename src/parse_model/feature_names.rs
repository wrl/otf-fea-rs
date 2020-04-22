use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    optional,
    value
};

use crate::parser::FeaRsStream;

use super::block::*;
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

pub(crate) fn name<Input, Ident>(_: &Ident) -> impl Parser<FeaRsStream<Input>, Output = Name>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("name")
        .skip(required_whitespace())
        .with(optional(
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
        ))

        .and(string())

        .map(|(ids, name)| {
            let (platform_id, ids) = ids.unwrap_or((Platform::Windows, None));

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

#[derive(Debug)]
pub struct FeatureNames {
    pub names: Vec<Name>
}

#[derive(Clone, PartialEq)]
pub struct NoIdent;

impl fmt::Display for NoIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

fn no_ident<Input>() -> impl Parser<FeaRsStream<Input>, Output = NoIdent>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    value(NoIdent)
}

pub(crate) fn feature_names<Input>() -> impl Parser<FeaRsStream<Input>, Output = FeatureNames>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("featureNames")
        .skip(required_whitespace())
        .with(block(no_ident, name))
        .map(|block| FeatureNames {
            names: block.statements
        })
}
