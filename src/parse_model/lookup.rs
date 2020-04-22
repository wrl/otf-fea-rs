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
use super::block::*;
use super::glyph::*;
use super::util::*;

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

#[inline]
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

        .with(block(lookup_block_label, block_statement))

        .map(|block|
            LookupDefinition {
                label: block.ident,
                statements: block.statements
            })
}

#[derive(Debug)]
pub struct Lookup(pub LookupBlockLabel);

pub(crate) fn lookup<Input>() -> impl Parser<FeaRsStream<Input>, Output = Lookup>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("lookup")
        .skip(required_whitespace())
        .with(lookup_block_label())
        .map(|label| Lookup(label))
}

pub(crate) enum LookupRefOrDefinition {
    Reference(Lookup),
    Definition(LookupDefinition)
}

impl LookupRefOrDefinition {
    pub fn parse<Input>() -> impl Parser<FeaRsStream<Input>, Output = Self>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        literal_ignore_case("lookup")
            .skip(required_whitespace())
            .with(block_or_reference(lookup_block_label, block_statement))
            .map(|res| {
                match res {
                    BlockOrReference::Block(block) =>
                        Self::Definition(LookupDefinition {
                            label: block.ident,
                            statements: block.statements
                        }),
                    BlockOrReference::Reference(r) => Self::Reference(Lookup(r))
                }
            })
    }
}
