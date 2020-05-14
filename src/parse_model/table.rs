use std::fmt;

use combine::{
    Parser,
    Stream,
    error::ParseError,

    dispatch,
    satisfy,
    value,
    token,

    parser::repeat::{
        sep_by,
        many1
    }
};

use crate::parser::FeaRsStream;

use super::block::*;
use super::glyph_class::*;
use super::util::*;

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TableTag {
    GDEF,
    head
}

impl fmt::Display for TableTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TableTag::*;

        match *self {
            GDEF => write!(f, "GDEF"),
            head => write!(f, "head")
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

#[derive(Debug)]
pub struct Attach {
    pub glyphs: GlyphClass,
    pub contour_points: Vec<usize>
}

#[derive(Debug)]
pub struct LigatureCaretByPos {
    pub glyphs: GlyphClass,
    pub carets: Vec<usize>
}

#[derive(Debug)]
pub struct LigatureCaretByIndex {
    pub glyphs: GlyphClass,
    pub carets: Vec<usize>
}

#[derive(Debug)]
pub struct FontRevision(f64);

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
    Attach(Attach),
    GlyphClassDef(GlyphClassDef),
    LigatureCaretByPos(LigatureCaretByPos),
    LigatureCaretByIndex(LigatureCaretByIndex),

    FontRevision(FontRevision)
}

cvt_to_statement!(Attach);
cvt_to_statement!(GlyphClassDef);
cvt_to_statement!(LigatureCaretByPos);
cvt_to_statement!(LigatureCaretByIndex);
cvt_to_statement!(FontRevision);

#[derive(Debug)]
pub struct Table {
    pub tag: TableTag,
    pub statements: Vec<TableStatement>
}

fn gdef_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    fn gcdef_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        sep_by(glyph_class_or_class_ref(),
                required_whitespace())
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
    }

    fn attach_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        glyph_class_or_glyph()
            .skip(required_whitespace())
            .and(many1(uinteger()
                    .skip(optional_whitespace())))
            .map(|(glyphs, contour_points)| Attach {
                glyphs,
                contour_points
            }.into())
    }

    fn ligature_caret<Input>() -> impl Parser<FeaRsStream<Input>, Output = (GlyphClass, Vec<usize>)>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        glyph_class_or_glyph()
            .skip(required_whitespace())
            .and(many1(uinteger()
                    .skip(optional_whitespace())))
    }

    combine::position()
        .and(keyword())
        .skip(required_whitespace())
        .then(|(position, kwd)| {
            dispatch!(&*kwd;
                "GlyphClassDef" => gcdef_statement(),
                "Attach" => attach_statement(),

                "LigatureCaretByPos" => ligature_caret()
                    .map(|(glyphs, carets)| LigatureCaretByPos {
                        glyphs, carets
                    }.into()),

                "LigatureCaretByIndex" => ligature_caret()
                    .map(|(glyphs, carets)| LigatureCaretByIndex {
                        glyphs, carets
                    }.into()),

                _ => value(position)
                .flat_map(|position|
                    crate::parse_bail!(Input, position,
                        "unexpected keyword"))
            )
        })
}

fn head_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    fn font_revision<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        decimal_number()
            .map(|fr| FontRevision(fr).into())
    }

    combine::position()
        .and(keyword())
        .skip(required_whitespace())
        .then(|(position, kwd)| {
            dispatch!(&*kwd;
                "FontRevision" => font_revision(),

                _ => value(position)
                .flat_map(|position|
                    crate::parse_bail!(Input, position,
                        "unexpected keyword"))
            )
        })
}


fn table_statement<Input>(tag: &TableTag) -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    dispatch!(tag;
        &TableTag::GDEF => gdef_statement(),
        &TableTag::head => head_statement()
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
