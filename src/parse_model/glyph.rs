use std::fmt;

use ascii::{
    AsciiChar,
    ToAsciiChar
};

use combine::{
    Parser,
    Stream,
    error::ParseError,

    parser,
    satisfy,
    choice,
    token
};

use arrayvec::ArrayVec;

use crate::parser::FeaRsStream;
use super::util::*;

pub(crate) type GlyphNameStorage = ArrayVec::<[AsciiChar; 63]>;

#[derive(Clone)]
pub struct GlyphName(pub GlyphNameStorage);

impl fmt::Debug for GlyphName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlyphName(\"")?;

        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        write!(f, "\")")
    }
}

#[inline]
pub(crate) fn glyph_character_valid(c: u8, first_character: bool, development_names: bool) -> bool
{
    match c {
        (b'a' ..= b'z') | (b'A' ..= b'Z') | b'_' => true,
        b'.' | (b'0' ..= b'9') if !first_character => true,

        b'*' | b'+' | b'-' | b':' | b'^' | b'|' | b'~'
            if development_names => true,

        _ => false
    }
}

pub(crate) fn glyph_name_unwrapped<Input>() -> impl Parser<FeaRsStream<Input>, Output = GlyphNameStorage>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    parser(move |input: &mut FeaRsStream<Input>| {
        let mut ret = ArrayVec::new();

        let development_names = input.state.development_glyph_names;

        let first_char = {
            use combine::ParseResult::*;

            let mut parser = satisfy(|c|
                glyph_character_valid(c, true, development_names));

            match parser.parse_stream(input) {
                CommitOk(ch) => ch,

                PeekOk(_) => {
                    // shouldn't happen?
                    panic!();
                },

                err => {
                    return err
                        .map(|_| ret)
                        .into();
                },
            }
        };

        let mut parse_iter =
            satisfy(|c| glyph_character_valid(c, false, development_names))
            .iter(input);

        let mut iter = std::iter::once(first_char)
            .chain(&mut parse_iter)
            .take(63);

        for c in &mut iter {
            // unsafe is fine here since we've already verified the character is in the
            // valid range via glyph_character_valid()

            ret.push(unsafe {
                (c as u8).to_ascii_char_unchecked()
            });
        }

        parse_iter.into_result(ret)
    })
    .expected("glyph name")
}

#[derive(Clone)]
pub struct GlyphCID(pub usize);

impl fmt::Debug for GlyphCID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlyphCID({})", self.0)
    }
}

#[derive(Clone)]
pub enum GlyphRef {
    Name(GlyphName),
    CID(GlyphCID)
}

impl fmt::Debug for GlyphRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlyphRef::Name(ref name) => {
                write!(f, "GlyphRef(name = \"")?;

                for c in &name.0 {
                    write!(f, "{}", c)?;
                }

                write!(f, "\")")
            },

            GlyphRef::CID(ref cid) =>
                write!(f, "GlyphRef(CID = {})", cid.0)
        }
    }
}

pub(crate) fn glyph_ref<Input>() -> impl Parser<FeaRsStream<Input>, Output = GlyphRef>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    choice((
        token(b'\\')
            .with(choice((
                uinteger()
                    .map(|cid| GlyphRef::CID(GlyphCID(cid))),

                glyph_name_unwrapped()
                    .map(|name| GlyphRef::Name(GlyphName(name)))
            ))),

        glyph_name_unwrapped().map(|name| GlyphRef::Name(GlyphName(name)))
    ))
}
