use std::{
    fmt,
    cmp
};

use ascii::{
    AsciiChar,
    ToAsciiChar
};

use combine::{
    Parser,
    Stream,
    error::ParseError,

    optional,
    satisfy
};

use crate::parser::FeaRsStream;
use super::glyph::*;

#[derive(Eq, Clone)]
pub struct Tag(pub [AsciiChar; 4]);

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tag(\"{}{}{}{}\")",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl cmp::PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(other.0.iter())
            .all(|(a, b)| a == b)
    }
}


pub(crate) fn tag<Input>() -> impl Parser<FeaRsStream<Input>, Output = Tag>
    where Input: Stream<Token = u8>,
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

            Tag(tag)
        })
}
