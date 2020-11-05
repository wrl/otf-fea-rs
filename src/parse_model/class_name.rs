use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    token,
};

use crate::parser::*;
use crate::glyph::*;

use super::glyph::*;

pub struct ClassName(pub GlyphNameStorage);

impl fmt::Debug for ClassName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClassName(\"@")?;

        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        write!(f, "\")")
    }
}

#[inline]
pub(crate) fn class_name<Input>() -> impl Parser<FeaRsStream<Input>, Output = ClassName>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    token(b'@')
        .with(glyph_name_unwrapped())
        .map(ClassName)
}
