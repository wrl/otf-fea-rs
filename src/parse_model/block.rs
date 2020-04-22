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
use super::feature_names::*;
use super::glyph_class::*;
use super::lookup_flag::*;
use super::mark_class::*;
use super::parameters::*;
use super::substitute::*;
use super::position::*;
use super::language::*;
use super::feature::*;
use super::lookup::*;
use super::script::*;

#[derive(Debug)]
pub enum BlockStatement {
    FeatureNames(FeatureNames),
    FeatureReference(FeatureReference),
    Language(Language),
    Lookup(Lookup),
    LookupDefinition(LookupDefinition),
    LookupFlag(LookupFlag),
    MarkClass(MarkClass),
    NamedGlyphClass(NamedGlyphClass),
    Parameters(Parameters),
    Position(Position),
    Script(Script),
    Substitute(Substitute),

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

cvt_to_statement!(FeatureNames);
cvt_to_statement!(FeatureReference);
cvt_to_statement!(Language);
cvt_to_statement!(Lookup);
cvt_to_statement!(LookupDefinition);
cvt_to_statement!(LookupFlag);
cvt_to_statement!(MarkClass);
cvt_to_statement!(NamedGlyphClass);
cvt_to_statement!(Parameters);
cvt_to_statement!(Position);
cvt_to_statement!(Script);
cvt_to_statement!(Substitute);

pub(crate) fn block_statement<Input, Ident>(_: &Ident) -> FnOpaque<FeaRsStream<Input>, BlockStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    #[inline]
    fn rule<Input>() -> impl Parser<FeaRsStream<Input>, Output = BlockStatement>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        choice((
                // if we have a preceding "ignore" statement, we'll skip over it in a look_ahead()
                // so that the statement for which it's relevant (substitute or position) can parse
                // it directly.

                look_ahead(
                    literal_ignore_case("ignore")
                        .or(literal_ignore_case("enum")
                            .skip(optional(literal_ignore_case("erate"))))
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

                    "feature" => feature_reference().map(|f| f.into()),
                    "lookupflag" => lookup_flag().map(|lf| lf.into()),
                    "markClass" => mark_class().map(|mc| mc.into()),
                    "script" => script().map(|s| s.into()),
                    "language" => language().map(|l| l.into()),

                    "featureNames" => feature_names().map(|n| n.into()),

                    "subtable" => literal("subtable").map(|_| BlockStatement::Subtable),

                    _ => combine::position().and(keyword())
                        .flat_map(|(position, kwd)|
                            crate::parse_bail!(Input, position,
                                format!("unexpected keyword \"{}\"", kwd))
                        )
                )
            })
    }

    // opaque is necessary here because otherwise we end up with this ugly recursive type since
    // blocks can be nested inside blocks

    opaque!(no_partial(
        choice((
            named_glyph_class()
                .expected("glyph class definition")
                .map(|gc| gc.into()),
            rule()
        ))
    ))
}

#[derive(Debug)]
pub struct Block<Ident, Statement> {
    pub ident: Ident,
    pub statements: Vec<Statement>
}

#[derive(Debug)]
pub enum BlockOrReference<Ident, Statement> {
    Block(Block<Ident, Statement>),
    Reference(Ident)
}

#[inline]
pub(crate) fn block_statements<Input, Statement, P>(statement_parser: P)
        -> impl Parser<FeaRsStream<Input>, Output = Vec<Statement>>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          P: Parser<FeaRsStream<Input>, Output = Statement>
{
    optional_whitespace()
        .with(many(
            optional_whitespace()
                .with(statement_parser)
                .skip(optional_whitespace())
                .skip(token(b';').expected("semicolon"))
                .skip(optional_whitespace())))
}

pub(crate) fn block_or_reference<Input, Ident, IF, IP, Statement, SF, SP>
            (ident_parser: IF, statement_parser: SF)
        -> impl Parser<FeaRsStream<Input>, Output = BlockOrReference<Ident, Statement>>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          IF: Fn() -> IP,
          IP: Parser<FeaRsStream<Input>, Output = Ident>,
          SF: Fn(&Ident) -> SP,
          SP: Parser<FeaRsStream<Input>, Output = Statement>,
          Ident: PartialEq + fmt::Display
{
    ident_parser()
        .skip(optional_whitespace())

        // FIXME: there's really no reason to "thread" ident through with a clone like this. it
        //        should be possible to have a variant of `then` which only passes an immutable
        //        ref in and then forwards its input through.
        .then_ref(move |ident| {
            optional(
                between(
                    token(b'{').expected("'{'"),
                    token(b'}').expected("'}'"),
                    block_statements(statement_parser(ident)))
                .skip(optional_whitespace())
                .and(combine::position()
                    .and(ident_parser())))
        })

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

#[inline]
pub(crate) fn block<Input, Ident, IF, IP, Statement, SF, SP>
            (ident_parser: IF, statement_parser: SF)
        -> impl Parser<FeaRsStream<Input>, Output = Block<Ident, Statement>>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          IF: Fn() -> IP,
          IP: Parser<FeaRsStream<Input>, Output = Ident>,
          SF: Fn(&Ident) -> SP,
          SP: Parser<FeaRsStream<Input>, Output = Statement>,
          Ident: PartialEq + fmt::Display
{
    combine::position()
        .and(block_or_reference(ident_parser, statement_parser))
        .flat_map(|(position, res)|
            match res {
                BlockOrReference::Block(b) => Ok(b),
                BlockOrReference::Reference(_) =>
                    crate::parse_bail!(Input, position,
                        "expected block")
            })
}
