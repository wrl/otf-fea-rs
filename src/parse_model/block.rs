use std::fmt;

use combine::{
    Parser,
    Stream,

    look_ahead,

    error::ParseError,

    between,
    token,

    parser::repeat::{
        many,
        many1
    },

    parser::byte::letter,

    dispatch
};

use crate::parser::FeaRsStream;

use super::util::*;
use super::parameters::*;
use super::substitute::*;
use super::position::*;
use super::lookup::*;

#[derive(Debug)]
pub enum BlockStatement {
    Parameters(Parameters),
    Substitute(Substitute),
    Position(Position),
    Lookup(Lookup),

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

cvt_to_statement!(Substitute);
cvt_to_statement!(Parameters);
cvt_to_statement!(Position);
cvt_to_statement!(Lookup);

#[inline]
fn keyword<Input>() -> impl Parser<FeaRsStream<Input>, Output = String>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    // from_utf8_unchecked() is safe here because letter() only matches ASCII chars.
    many1(letter()).map(|x| unsafe { String::from_utf8_unchecked(x) })
}

pub(crate) fn block_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = BlockStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    look_ahead(keyword())
        .then(|kwd| {
            dispatch!(&*kwd;
                "parameters" => parameters().map(|p| p.into()),
                "position" | "pos" => position().map(|p| p.into()),
                "substitute" | "sub"
                    | "reversesub" | "rsub" => substitute().map(|s| s.into()),
                "lookup" => lookup().map(|l| l.into()),

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
}

#[derive(Debug)]
pub struct Block<Ident> {
    pub ident: Ident,
    pub statements: Vec<BlockStatement>
}

pub(crate) fn block<Input, Ident, F, P>(ident_parser: F)
        -> impl Parser<FeaRsStream<Input>, Output = Block<Ident>>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          F: Fn() -> P,
          P: Parser<FeaRsStream<Input>, Output = Ident>,
          Ident: PartialEq + fmt::Display
{
    ident_parser()
        .skip(optional_whitespace())

        .and(between(token(b'{').expected("'{'"), token(b'}').expected("'}'"),
            optional_whitespace()
                .with(many(
                    optional_whitespace()
                        .with(block_statement())
                        .skip(optional_whitespace()))
                )))

        .skip(optional_whitespace())
        .and(combine::position())
        .and(ident_parser())

        .flat_map(|(((ident, statements), position), closing_ident)| {
            if ident != closing_ident {
                crate::parse_bail!(Input, position,
                    format!("mismatched block identifier (starting \"{}\", closing\"{}\")",
                            ident, closing_ident));
            }

            Ok(Block {
                ident,
                statements
            })})
}
