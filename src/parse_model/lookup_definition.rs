use std::{
    fmt,
    cmp
};

use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::FeaRsStream;

use super::util::*;
use super::block::*;
use super::glyph::*;

#[derive(cmp::PartialEq)]
pub struct LookupBlockLabel(pub GlyphNameStorage);

impl fmt::Debug for LookupBlockLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LookupBlockLabel(\"")?;

        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        write!(f, "\")")
    }
}

impl fmt::Display for LookupBlockLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        Ok(())
    }
}

pub(crate) fn lookup_block_label<Input>() -> impl Parser<FeaRsStream<Input>, Output = LookupBlockLabel>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    glyph_name_unwrapped()
        .map(|gn| LookupBlockLabel(gn))
}

#[derive(Debug)]
pub struct LookupDefinition {
    pub label: LookupBlockLabel,
    pub statements: Vec<BlockStatement>
}

pub(crate) fn lookup_definition<Input>() -> impl Parser<FeaRsStream<Input>, Output = LookupDefinition>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("lookup")
        .with(required_whitespace())

        .with(block(lookup_block_label))

        .map(|block|
            LookupDefinition {
                label: block.ident,
                statements: block.statements
            })
}
