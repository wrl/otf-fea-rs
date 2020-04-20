use std::fmt;

use combine::{
    Parser,
    Stream,

    look_ahead,
    optional,

    opaque,
    parser::combinator::{
        FnOpaque,
        no_partial
    },

    error::ParseError,

    between,
    token,

    parser::repeat::many,

    choice,
    dispatch
};

use crate::parser::FeaRsStream;

use super::util::*;
use super::lookup_flag::*;
use super::mark_class::*;
use super::parameters::*;
use super::substitute::*;
use super::position::*;
use super::language::*;
use super::lookup::*;
use super::script::*;

#[derive(Debug)]
pub enum BlockStatement {
    LookupDefinition(LookupDefinition),
    LookupFlag(LookupFlag),
    Parameters(Parameters),
    Substitute(Substitute),
    MarkClass(MarkClass),
    Language(Language),
    Position(Position),
    Lookup(Lookup),
    Script(Script),

    Subtable
}

macro_rules! cvt_to_statement (
    ($iden:ident) => {
        impl From<$iden> for BlockStatement {
            fn from(x: $iden) -> BlockStatement {
                BlockStatement::$iden(x)
            }
        }
    }
);

cvt_to_statement!(LookupDefinition);
cvt_to_statement!(LookupFlag);
cvt_to_statement!(Parameters);
cvt_to_statement!(Substitute);
cvt_to_statement!(MarkClass);
cvt_to_statement!(Language);
cvt_to_statement!(Position);
cvt_to_statement!(Lookup);
cvt_to_statement!(Script);

pub(crate) fn block_statement<Input>() -> FnOpaque<FeaRsStream<Input>, BlockStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    // opaque is necessary here because otherwise we end up with this ugly recursive type since
    // blocks can be nested inside blocks

    opaque!(no_partial(
        choice((
                look_ahead(
                    literal_ignore_case("ignore")
                        .skip(required_whitespace())
                        .with(keyword())),
                look_ahead(keyword()),
        ))
            .then(|kwd| {
                dispatch!(&*kwd;
                    "parameters" => parameters().map(|p| p.into()),
                    "position" | "pos" => position().map(|p| p.into()),
                    "substitute" | "sub"
                        | "reversesub" | "rsub" => substitute().map(|s| s.into()),

                    "lookup" =>
                        LookupRefOrDefinition::parse()
                        .map(|x| match x {
                            LookupRefOrDefinition::Definition(d) => d.into(),
                            LookupRefOrDefinition::Reference(r) => r.into()
                        }),

                    "lookupflag" => lookup_flag().map(|lf| lf.into()),
                    "markClass" => mark_class().map(|mc| mc.into()),
                    "script" => script().map(|s| s.into()),
                    "language" => language().map(|l| l.into()),

                    "subtable" => literal("subtable").map(|_| BlockStatement::Subtable),

                    _ => combine::position().and(keyword())
                        .flat_map(|(position, kwd)|
                            crate::parse_bail!(Input, position,
                                format!("unexpected keyword \"{}\"", kwd))
                        )
                )
            })

            .skip(optional_whitespace())
            .skip(token(b';').expected("semicolon"))
    ))
}

#[derive(Debug)]
pub struct Block<Ident> {
    pub ident: Ident,
    pub statements: Vec<BlockStatement>
}

#[derive(Debug)]
pub enum BlockOrReference<Ident> {
    Block(Block<Ident>),
    Reference(Ident)
}

#[inline]
pub(crate) fn block_statements<Input>()
        -> impl Parser<FeaRsStream<Input>, Output = Vec<BlockStatement>>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    optional_whitespace()
        .with(many(
                optional_whitespace()
                .with(block_statement())
                .skip(optional_whitespace())))
}

pub(crate) fn block_or_reference<Input, Ident, F, P>(ident_parser: F)
        -> impl Parser<FeaRsStream<Input>, Output = BlockOrReference<Ident>>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          F: Fn() -> P,
          P: Parser<FeaRsStream<Input>, Output = Ident>,
          Ident: PartialEq + fmt::Display
{
    ident_parser()
        .skip(optional_whitespace())

        .and(optional(
            between(
                token(b'{').expected("'{'"),
                token(b'}').expected("'}'"),
                block_statements())
            .skip(optional_whitespace())
            .and(combine::position()
                .and(ident_parser()))))

        .flat_map(|(opening_ident, block_innards)| {
            let res = match block_innards {
                None => BlockOrReference::Reference(opening_ident),
                Some((statements, (position, closing_ident))) => {
                    if opening_ident != closing_ident {
                        crate::parse_bail!(Input, position,
                            format!("mismatched block identifier (opening \"{}\", closing\"{}\")",
                            opening_ident, closing_ident));
                    }

                    BlockOrReference::Block(Block {
                        ident: opening_ident,
                        statements
                    })
                }
            };

            Ok(res)
        })
}

pub(crate) fn block<Input, Ident, F, P>(ident_parser: F)
        -> impl Parser<FeaRsStream<Input>, Output = Block<Ident>>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          F: Fn() -> P + Clone,
          P: Parser<FeaRsStream<Input>, Output = Ident>,
          Ident: PartialEq + fmt::Display
{
    combine::position()
        .and(block_or_reference(ident_parser))
        .flat_map(|(position, res)|
            match res {
                BlockOrReference::Block(b) => Ok(b),
                BlockOrReference::Reference(_) =>
                    crate::parse_bail!(Input, position,
                        "expected block")
            })
}

pub(crate) fn reference<Input, Ident, F, P>(ident_parser: F)
        -> impl Parser<FeaRsStream<Input>, Output = Ident>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          F: Fn() -> P + Clone,
          P: Parser<FeaRsStream<Input>, Output = Ident>,
          Ident: PartialEq + fmt::Display
{
    combine::position()
        .and(block_or_reference(ident_parser))
        .flat_map(|(position, res)|
            match res {
                BlockOrReference::Block(_) =>
                    crate::parse_bail!(Input, position,
                        "expected reference"),
                BlockOrReference::Reference(r) => Ok(r)
            })
}
