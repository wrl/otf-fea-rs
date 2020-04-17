use combine::{
    Parser,
    Stream,
    error::ParseError,

    parser,
    token,

    look_ahead,
    optional,

    choice
};

use crate::parser::FeaRsStream;
use super::value_record::*;
use super::glyph_class::*;
use super::lookup::*;
use super::util::*;

#[derive(Debug)]
pub struct GlyphPatternItem {
    pub class: GlyphClass,
    pub value_record: Option<ValueRecord>,
    pub lookup: Option<Lookup>
}

#[derive(Debug, Default)]
pub struct GlyphPattern {
    pub prefix: Vec<GlyphPatternItem>,
    pub glyphs: Vec<GlyphPatternItem>,
    pub suffix: Vec<GlyphPatternItem>,

    pub has_marks: bool,

    pub num_value_records: usize,
    pub num_lookups: usize
}

impl GlyphPattern {
    fn new() -> Self {
        GlyphPattern {
            prefix: Vec::new(),
            glyphs: Vec::new(),
            suffix: Vec::new(),

            has_marks: false,

            num_value_records: 0,
            num_lookups: 0
        }
    }
}

// eats trailing whitespace because otherwise we'd have to lookahead *all* of the whitespace
pub(crate) fn glyph_pattern<Input>() -> impl Parser<FeaRsStream<Input>, Output = GlyphPattern>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    #[derive(Debug)]
    enum Next {
        GlyphClass(GlyphClass, Option<ValueRecord>),
        MarkedGlyphClass(GlyphClass, Option<ValueRecord>, Option<Lookup>),
        End
    }

    parser(|input| {
        let mut parse_iter = optional_whitespace()
            .with(choice!(
                peek(literal_ignore_case("by"))
                    .map(|_| Next::End),
                peek(literal_ignore_case("from"))
                    .map(|_| Next::End),
                look_ahead(token(b';'))
                    .map(|_| Next::End),
                look_ahead(token(b','))
                    .map(|_| Next::End),

                glyph_class_or_glyph()
                    .and(optional(token(b'\''))
                        .skip(optional_whitespace())
                        .and(optional(
                                peek(literal_ignore_case("lookup"))
                                    .with(lookup()))))
                    .skip(optional_whitespace())
                    .and(optional(value_record()))
                    .map(|((gc, (mark, lookup)), vr)|
                        match mark {
                            Some(_) => Next::MarkedGlyphClass(gc, vr, lookup),
                            None => Next::GlyphClass(gc, vr)
                        })
            ))
            .iter(input);

        let mut pattern = GlyphPattern::new();

        for next in &mut parse_iter {
            match next {
                Next::GlyphClass(gc, vr) => {
                    let item = GlyphPatternItem {
                        class: gc,
                        value_record: vr,
                        lookup: None
                    };

                    if pattern.glyphs.len() > 0 {
                        pattern.suffix.push(item);
                    } else {
                        pattern.prefix.push(item);
                    }
                },

                Next::MarkedGlyphClass(gc, vr, lookup) => {
                    // FIXME: raise error if a second run of marked characters occurs.
                    //        i.e. if we're marked but there's already characters in suffix

                    pattern.has_marks = true;

                    if lookup.is_some() {
                        pattern.num_lookups += 1;
                    }

                    let item = GlyphPatternItem {
                        class: gc,
                        value_record: vr,
                        lookup,
                    };

                    pattern.glyphs.push(item);
                },

                Next::End => break,
            };
        }

        if pattern.glyphs.len() == 0 && pattern.suffix.len() == 0 {
            let pattern = GlyphPattern {
                prefix: Vec::new(),
                glyphs: pattern.prefix,
                suffix: Vec::new(),

                ..pattern
            };

            parse_iter.into_result(pattern)
        } else {
            parse_iter.into_result(pattern)
        }
    })
}
