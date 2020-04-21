use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    dispatch,
    satisfy,
    value,
    token,

    parser::repeat::sep_by
};

use crate::parser::FeaRsStream;

use super::block::*;
use super::glyph_class::*;
use super::util::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TableTag {
    GDEF
}

impl fmt::Display for TableTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TableTag::*;

        match *self {
            GDEF => write!(f, "GDEF")
        }
    }
}

#[derive(Debug)]
pub struct GlyphClassDef {
    pub base: Vec<GlyphClass>,
    pub ligature: Vec<GlyphClass>,
    pub mark: Vec<GlyphClass>,
    pub component: Vec<GlyphClass>
}

macro_rules! cvt_to_statement (
    ($iden:ident) => {
        impl From<$iden> for TableStatement {
            fn from(x: $iden) -> TableStatement {
                TableStatement::$iden(x)
            }
        }
    }
);

cvt_to_statement!(GlyphClassDef);

#[derive(Debug)]
pub enum TableStatement {
    GlyphClassDef(GlyphClassDef)
}

#[derive(Debug)]
pub struct Table {
    pub tag: TableTag,
    pub statements: Vec<TableStatement>
}

fn gdef_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(keyword())
        .then(|(position, kwd)| {
            dispatch!(&*kwd;
                "GlyphClassDef" => {
                    required_whitespace()
                        .with(sep_by(glyph_class_or_class_ref(),
                                required_whitespace()))
                        .skip(optional_whitespace())
                        .skip(token(b','))
                        .skip(optional_whitespace())
                        .and(sep_by(glyph_class_or_class_ref(),
                                required_whitespace()))
                        .skip(optional_whitespace())
                        .skip(token(b','))
                        .skip(optional_whitespace())
                        .and(sep_by(glyph_class_or_class_ref(),
                                required_whitespace()))
                        .skip(optional_whitespace())
                        .skip(token(b','))
                        .skip(optional_whitespace())
                        .and(sep_by(glyph_class_or_class_ref(),
                                required_whitespace()))
                        .map(|(((base, ligature), mark), component)| GlyphClassDef {
                            base,
                            ligature,
                            mark,
                            component
                        }.into())
                },

                _ => value(position)
                .flat_map(|position|
                    crate::parse_bail!(Input, position,
                        "unexpected keyword"))
            )
        })
}


fn table_statement<Input>(_tag: &TableTag) -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    gdef_statement()
        .skip(optional_whitespace())
        .skip(token(b';').expected("semicolon"))
}

fn table_tag<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableTag>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    fn char_valid(x: u8) -> bool {
        let x = x as char;

        x.is_ascii_alphabetic() || x == '/'
    }

    combine::position()
        .and(satisfy(|x| char_valid(x)))
        .and(satisfy(|x| char_valid(x)))
        .and(satisfy(|x| char_valid(x)))
        .and(satisfy(|x| char_valid(x)))
        .flat_map(|((((position, one), two), three), four)| {
            let tag = &[one, two, three, four];

            Ok(match tag {
                b"GDEF" => TableTag::GDEF,

                _ =>
                    crate::parse_bail!(Input, position,
                        "unknown table identifier")
            })
        })
}

pub(crate) fn table<Input>() -> impl Parser<FeaRsStream<Input>, Output = Table>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("table")
        .skip(required_whitespace())

        .with(block(table_tag, table_statement))

        .map(|block|
            Table {
                tag: block.ident,
                statements: block.statements
            })
}
