use ascii::{
    AsciiChar,
    ToAsciiChar,
};

use combine::{
    Parser,
    Stream,
    error::ParseError,

    optional,
    satisfy
};

use crate::tag::*;
use crate::glyph::*;

use crate::parser::*;


pub(crate) fn tag_storage<Input>() -> impl Parser<FeaRsStream<Input>, Output = TagStorage>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    satisfy(|c| glyph_character_valid(c, true, false))
        .and(optional(satisfy(|c| glyph_character_valid(c, false, false))))
        .and(optional(satisfy(|c| glyph_character_valid(c, false, false))))
        .and(optional(satisfy(|c| glyph_character_valid(c, false, false))))
        .map(|(((one, two), three), four): (((u8, Option<u8>), Option<u8>), Option<u8>)| {
            let mut tag = [AsciiChar::Space; 4];

            tag[0] = unsafe { one.to_ascii_char_unchecked() };
            two.map(|x| tag[1] = unsafe { x.to_ascii_char_unchecked() });
            three.map(|x| tag[2] = unsafe { x.to_ascii_char_unchecked() });
            four.map(|x| tag[3] = unsafe { x.to_ascii_char_unchecked() });

            tag
        })
}

pub(crate) fn tag<Input>() -> impl Parser<FeaRsStream<Input>, Output = Tag>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    tag_storage()
        .map(Tag)
}
