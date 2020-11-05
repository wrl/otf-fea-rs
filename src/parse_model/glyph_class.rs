use combine::{
    Parser,
    Stream,
    error::ParseError,

    parser,
    choice,
    token
};

use crate::parser::*;
use crate::glyph_class::*;

use super::class_name::*;
use super::glyph::*;
use super::util::*;


pub(crate) fn glyph_class<Input>() -> impl Parser<FeaRsStream<Input>, Output = GlyphClass>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    // alright, this one gets ugly.
    // there are two possible forms here:
    //
    //     list of glyphs: [a b c ...]
    //     range of glyphs: [a - b]
    //
    // there are a few notes in the specification that make this a pain to parse. first, when
    // specifying a glyph range, the whitespace around the hyphen is optional, which means that we
    // can't just parse one glyph reference, whitespace, and then see if we have a hyphen or
    // another glyph name. we have to handle
    //
    //      [a-  -> range
    //      [a - -> range
    //      [a b -> list of glyphs
    //
    // with the combinators we have, there's a few ugly branches we have to do, but it's nothing
    // too awful. just ugly control flow from here on out.
    //
    // BONUS UGLINESS: the above only comes into account because (from the spec) "hyphens are not
    // permitted in feature file glyph names". however, they *are* permitted in develpment glyph
    // names. so, with development glyph names, [abcd-efgh] is a class consisting of a singular
    // glyph, "abcd-efgh".

    enum Next<T> {
        Glyph(T),
        ClassRef(GlyphClassName),
        RangeSpec(T),
        EndClass
    }

    token(b'[')
        .with(parser(|input| {
                let mut glyphs = vec![];

                let mut parse_iter = optional_whitespace()
                    .with(combine::position())
                    .and(choice((
                        token(b'-')
                            .skip(optional_whitespace())
                            .with(glyph_ref()).map(|gr| Next::RangeSpec(gr)),

                        glyph_class_name()
                            .map(|cn| Next::ClassRef(cn)),

                        glyph_ref()
                            .map(|gr| Next::Glyph(gr)),

                        token(b']')
                            .map(|_| Next::EndClass)
                    )))
                    .iter(input);

                for (_, next) in &mut parse_iter {
                    match next {
                        Next::ClassRef(class) =>
                            glyphs.push(GlyphClassItem::ClassRef(class)),

                        Next::Glyph(glyph) => glyphs.push(GlyphClassItem::Single(glyph)),

                        Next::RangeSpec(end) => {
                            match glyphs.pop() {
                                Some(GlyphClassItem::Single(start)) => {
                                    glyphs.push(GlyphClassItem::Range {
                                        start,
                                        end
                                    });
                                },

                                // FIXME: better errors here

                                Some(GlyphClassItem::Range { .. }) => {
                                    // [a - b - c]
                                    // think this illegal but who knows
                                    panic!();
                                },

                                Some(GlyphClassItem::ClassRef(_)) => {
                                    panic!();
                                }

                                None => {
                                    // [- b]
                                    // tried to define the end of a range at the start of the class
                                    panic!();
                                }
                            }
                        },

                        Next::EndClass => break
                    }
                }

                parse_iter.into_result(GlyphClass(glyphs))
        }))
}

pub(crate) fn glyph_class_or_class_ref<Input>() -> impl Parser<FeaRsStream<Input>, Output = GlyphClass>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    choice((
        glyph_class().map(|gc| gc),
        glyph_class_name().map(|cn|
            GlyphClass(vec![GlyphClassItem::ClassRef(cn)]))
    ))
}

pub(crate) fn glyph_class_or_glyph<Input>() -> impl Parser<FeaRsStream<Input>, Output = GlyphClass>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    choice((
        glyph_class().map(|gc| gc),
        glyph_ref().map(|gr| GlyphClass::from_single(gr)),
        glyph_class_name().map(|cn|
            GlyphClass(vec![GlyphClassItem::ClassRef(cn)]))
    ))
}

/////////////////////////
// named glyph classes
/////////////////////////

pub(crate) fn glyph_class_name<Input>() -> impl Parser<FeaRsStream<Input>, Output = GlyphClassName>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    class_name()
        .map(|cn| GlyphClassName(cn.0))
}

pub(crate) fn named_glyph_class<Input>() -> impl Parser<FeaRsStream<Input>, Output = NamedGlyphClass>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    class_name()
        .map(|cn| GlyphClassName(cn.0))
        .skip(optional_whitespace())
        .skip(token(b'='))
        .skip(optional_whitespace())
        .and(glyph_class())

        .map(|(name, glyph_class)| {
            NamedGlyphClass {
                name,
                glyph_class
            }
        })
}
