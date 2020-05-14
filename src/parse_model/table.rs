use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    dispatch,
    satisfy
};

use crate::parser::FeaRsStream;

use crate::parse_model::block::*;
use crate::parse_model::util::*;

use crate::parse_model::tables::gdef::*;
use crate::parse_model::tables::head::*;
use crate::parse_model::tables::hhea::*;

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TableTag {
    GDEF,
    head,
    hhea
}

impl fmt::Display for TableTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TableTag::*;

        match *self {
            GDEF => write!(f, "GDEF"),
            head => write!(f, "head"),
            hhea => write!(f, "hhea")
        }
    }
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

#[derive(Debug)]
pub enum TableStatement {
    // GDEF
    Attach(Attach),
    GlyphClassDef(GlyphClassDef),
    LigatureCaretByPos(LigatureCaretByPos),
    LigatureCaretByIndex(LigatureCaretByIndex),

    // head
    FontRevision(FontRevision),

    // hhea
    CaretOffset(CaretOffset),
    Ascender(Ascender),
    Descender(Descender),
    LineGap(LineGap)
}

cvt_to_statement!(Attach);
cvt_to_statement!(GlyphClassDef);
cvt_to_statement!(LigatureCaretByPos);
cvt_to_statement!(LigatureCaretByIndex);
cvt_to_statement!(FontRevision);
cvt_to_statement!(CaretOffset);
cvt_to_statement!(Ascender);
cvt_to_statement!(Descender);
cvt_to_statement!(LineGap);

#[derive(Debug)]
pub struct Table {
    pub tag: TableTag,
    pub statements: Vec<TableStatement>
}

fn table_statement<Input>(tag: &TableTag) -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    dispatch!(tag;
        &TableTag::GDEF => gdef_statement(),
        &TableTag::head => head_statement(),
        &TableTag::hhea => hhea_statement()
    )
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
        .and(satisfy(char_valid))
        .and(satisfy(char_valid))
        .and(satisfy(char_valid))
        .and(satisfy(char_valid))
        .flat_map(|((((position, one), two), three), four)| {
            let tag = &[one, two, three, four];

            Ok(match tag {
                b"GDEF" => TableTag::GDEF,
                b"head" => TableTag::head,
                b"hhea" => TableTag::hhea,

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
