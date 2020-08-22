use combine::{
    Parser,
    Stream,
    error::ParseError,

    dispatch,
    value,
    token,

    parser::repeat::{
        sep_by,
        many1
    }
};

use crate::parser::FeaRsStream;
use crate::glyph_class::*;

use crate::parse_model::glyph_class::*;
use crate::parse_model::table::*;
use crate::parse_model::util::*;

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

pub(crate) fn gdef_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TableStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
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
