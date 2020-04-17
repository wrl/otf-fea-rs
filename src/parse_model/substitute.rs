use combine::{
    Parser,
    Stream,
    error::ParseError,

    value,

    parser::repeat::many1,
    parser::byte::letter,

    choice
};

use crate::parser::FeaRsStream;
use super::glyph_pattern::*;
use super::glyph_class::*;
use super::glyph::*;
use super::util::*;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Substitute {
    // GSUB type 1
    Single {
        prefix: Vec<GlyphClass>,
        glyph_class: GlyphClass,
        suffix: Vec<GlyphClass>,
        replacement: GlyphClass,

        force_chain: bool
    },

    // GSUB type 2
    Multiple {
        glyph: GlyphRef,
        sequence: Vec<GlyphRef>
    },

    // GSUB type 3
    Alternate {
        prefix: Vec<GlyphClass>,
        glyph: GlyphClass,
        suffix: Vec<GlyphClass>,
        replacement: GlyphClass
    },
}

#[inline]
fn into_glyphs(items: Vec<GlyphPatternItem>) -> Vec<GlyphClass>
{
    items.into_iter().map(|g| g.class).collect()
}

#[inline]
fn into_first<T>(v: Vec<T>) -> Option<T>
{
    v.into_iter().next()
}

#[inline]
fn into_first_glyph(items: Vec<GlyphPatternItem>) -> Option<GlyphClass>
{
    into_first(items).map(|g| g.class)
}

pub(crate) fn substitute<Input>() -> impl Parser<FeaRsStream<Input>, Output = Substitute>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    #[derive(PartialEq, Eq)]
    enum SubKeyword {
        By,
        From
    }

    #[derive(PartialEq, Eq)]
    enum Direction {
        Forward,
        Reverse
    }

    combine::position().and(many1(letter()))
        .flat_map(|(position, kwd): (_, Vec<u8>)| {
            let direction = match &*kwd {
                b"substitute" | b"sub" => Direction::Forward,
                b"reversesub" | b"rsub" => Direction::Reverse,
                _ => crate::parse_bail!(Input, position, "unexpected keyword")
            };

            Ok((position, direction))
        })
        .skip(required_whitespace())
        .and(glyph_pattern())
        .and(choice((
            literal("by").map(|_| Some(SubKeyword::By))
                .and(many1(
                    required_whitespace()
                    .with(glyph_class_or_glyph()))),
            literal("from").map(|_| Some(SubKeyword::From))
                .skip(required_whitespace())
                .and(glyph_class_or_class_ref().map(|gc| vec![gc])),
            value(()).map(|_| (None, vec![]))
        )))
        .flat_map(|(((position, direction), pattern), (keyword, replacement))| {
            if pattern.num_value_records > 0 {
                crate::parse_bail!(Input, position,
                    "Substitution statements cannot contain value records");
            }

            // GSUB lookup type 3
            //     "substitute a from [a.1 a.2 a.3];"
            if keyword == Some(SubKeyword::From) {
                if direction == Direction::Reverse {
                    crate::parse_bail!(Input, position,
                        "Reverse chaining substitutions do not support \"from\"");
                }

                if pattern.glyphs.len() != 1 || pattern.glyphs[0].class.0.len() != 1 {
                    crate::parse_bail!(Input, position,
                        "Expected a single glyph before \"from\"");
                }

                if replacement.len() != 1 {
                    crate::parse_bail!(Input, position,
                        "Expected a single glyph class after \"from\"");
                }

                return Ok(Substitute::Alternate {
                    prefix: into_glyphs(pattern.prefix),
                    glyph: into_first_glyph(pattern.glyphs).unwrap(),
                    suffix: into_glyphs(pattern.suffix),
                    replacement: into_first(replacement).unwrap()
                });
            }

            // GSUB lookup type 1
            //     "substitute a by a.sc;"
            //     "substitute [one.fitted one.oldstyle] by one;"
            //     "substitute [a-d] by [A.sc-D.sc];"
            if direction == Direction::Forward
                    && pattern.glyphs.len() == 1 && replacement.len() == 1
                    && pattern.num_lookups == 0 {

                return Ok(Substitute::Single {
                    prefix: into_glyphs(pattern.prefix),
                    glyph_class: into_first_glyph(pattern.glyphs).unwrap(),
                    suffix: into_glyphs(pattern.suffix),
                    replacement: into_first(replacement).unwrap(),

                    force_chain: pattern.has_marks
                });
            }

            Ok(Substitute::Single {
                prefix: Vec::new(),
                glyph_class: GlyphClass(vec![]),
                suffix: Vec::new(),
                replacement: GlyphClass(vec![]),
                force_chain: false
            })
        })
}
