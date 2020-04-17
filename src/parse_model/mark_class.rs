use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::FeaRsStream;
use super::glyph_class::*;
use super::class_name::*;
use super::anchor::*;
use super::glyph::*;
use super::util::*;

pub struct MarkClassName(pub GlyphNameStorage);

impl fmt::Debug for MarkClassName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MarkClassName(\"@")?;

        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        write!(f, "\")")
    }
}

#[derive(Debug)]
pub struct MarkClass {
    pub glyph_class: GlyphClass,
    pub anchor: Anchor,
    pub class_name: MarkClassName
}

pub(crate) fn mark_class<Input>() -> impl Parser<FeaRsStream<Input>, Output = MarkClass>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("markClass")
        .skip(required_whitespace())
        .with(glyph_class_or_glyph())
        .skip(required_whitespace())
        .and(anchor())
        .skip(required_whitespace())
        .and(class_name())

        .map(|((glyph_class, anchor), class_name)| {
            MarkClass {
                glyph_class,
                anchor,
                class_name: MarkClassName(class_name.0)
            }
        })
}
