use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError
};

use crate::parser::FeaRsStream;
use crate::glyph::*;

use super::block::*;
use super::glyph::*;
use super::util::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LookupName(pub GlyphNameStorage);

impl fmt::Debug for LookupName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LookupName(\"")?;

        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        write!(f, "\")")
    }
}

impl fmt::Display for LookupName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.0 {
            write!(f, "{}", c)?;
        }

        Ok(())
    }
}

#[inline]
pub(crate) fn lookup_block_label<Input>() -> impl Parser<FeaRsStream<Input>, Output = LookupName>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    glyph_name_unwrapped()
        .map(LookupName)
}

#[derive(Debug)]
pub struct LookupDefinition {
    pub label: LookupName,
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
pub struct Lookup(pub LookupName);

pub(crate) fn lookup<Input>() -> impl Parser<FeaRsStream<Input>, Output = Lookup>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("lookup")
        .skip(required_whitespace())
        .with(lookup_block_label())
        .map(Lookup)
}

pub enum LookupRefOrDefinition {
    Reference(Lookup),
    Definition(LookupDefinition)
}

impl LookupRefOrDefinition {
    pub(crate) fn parse<Input>() -> impl Parser<FeaRsStream<Input>, Output = Self>
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
