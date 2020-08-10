use combine::{
    Parser,
    Stream,
    error::ParseError,

    parser,

    look_ahead,
    optional,

    parser::repeat::{
        take_until,
        many1
    },

    parser::byte::space,

    dispatch,
    choice,
    value
};

use crate::parser::FeaRsStream;
use super::value_record::*;
use super::glyph_class::*;
use super::class_name::*;
use super::mark_class::*;
use super::anchor::*;
use super::util::*;

#[derive(Debug)]
pub struct LigatureComponent {
    anchors: Vec<(Anchor, Option<MarkClassName>)>
}

#[derive(Debug)]
pub enum Position {
    // GPOS type 1
    SingleAdjustment {
        glyph_class: GlyphClass,
        value_record: ValueRecord
    },

    // GPOS type 2
    Pair {
        glyph_classes: [GlyphClass; 2],
        value_records: (ValueRecord, Option<ValueRecord>)
    },

    // GPOS type 3
    Cursive {
        glyph_class: GlyphClass,
        entry: Anchor,
        exit: Anchor
    },

    // GPOS type 4
    MarkToBase {
        glyph_class: GlyphClass,
        anchors: Vec<(Anchor, MarkClassName)>
    },

    // GPOS type 5
    Ligature {
        glyph_class: GlyphClass,
        components: Vec<LigatureComponent>
    },

    // GPOS type 6
    MarkToMark {
        glyph_class: GlyphClass,
        anchors: Vec<(Anchor, MarkClassName)>
    },

    // FIXME: more of these
}

fn cursive<Input>() -> impl Parser<FeaRsStream<Input>, Output = Position>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("cursive")
        .skip(required_whitespace())
        .with(glyph_class_or_glyph())
        .skip(required_whitespace())
        .and(anchor())
        .skip(required_whitespace())
        .and(anchor())

        .map(|((glyph_class, entry), exit)| {
            Position::Cursive {
                glyph_class,
                entry,
                exit
            }
        })
}

fn ligature<Input>() -> impl Parser<FeaRsStream<Input>, Output = Position>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    #[inline]
    pub(crate) fn ligature_component<Input>() -> impl Parser<FeaRsStream<Input>, Output = LigatureComponent>
        where Input: Stream<Token = u8>,
              Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
    {
        many1(
            anchor()
                .then_ref(|anchor| {
                    if let Anchor::Null = *anchor {
                        return optional_whitespace()
                            .map(|_| None)
                            .left();
                    }

                    required_whitespace()
                        .skip(literal_ignore_case("mark"))
                        .skip(required_whitespace())
                        .with(class_name()
                            .map(|cn| Some(MarkClassName(cn.0))))
                        .skip(optional_whitespace())
                        .right()
                }))
            .map(|anchors| LigatureComponent {
                anchors
            })
    }

    literal_ignore_case("ligature")
        .skip(required_whitespace())
        .with(glyph_class_or_glyph())
        .skip(required_whitespace())
        .and(parser(|input| {
            let mut components = {
                use combine::ParseResult::*;

                match ligature_component().parse_stream(input) {
                    CommitOk(lc) => vec![lc],
                    PeekOk(_) => panic!(),
                    err => return err.map(|_| vec![]).into()
                }
            };

            let mut parse_iter =
                literal("ligComponent")
                .skip(required_whitespace())
                .with(ligature_component())
                .iter(input);

            for c in &mut parse_iter {
                components.push(c);
            };

            parse_iter.into_result(components)
        }))

        .map(|(glyph_class, components)|
            Position::Ligature {
                glyph_class,
                components
            })
}

fn mark_to<Input>() -> impl Parser<FeaRsStream<Input>, Output = (GlyphClass, Vec<(Anchor, MarkClassName)>)>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    glyph_class_or_glyph()
        .and(many1(
            required_whitespace()
            .with(anchor())
            .skip(required_whitespace())
            .skip(literal_ignore_case("mark"))
            .skip(required_whitespace())
            .and(class_name()
                .map(|cn| MarkClassName(cn.0)))
            ))
}

fn mark_to_base<Input>() -> impl Parser<FeaRsStream<Input>, Output = Position>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("base")
        .skip(required_whitespace())

        .with(mark_to())
        .map(|(glyph_class, anchors)| {
            Position::MarkToBase {
                glyph_class,
                anchors
            }
        })
}

fn mark_to_mark<Input>() -> impl Parser<FeaRsStream<Input>, Output = Position>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("mark")
        .skip(required_whitespace())

        .with(mark_to())
        .map(|(glyph_class, anchors)| {
            Position::MarkToMark {
                glyph_class,
                anchors
            }
        })
}

fn single_or_pair<Input>() -> impl Parser<FeaRsStream<Input>, Output = Position>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    glyph_class_or_glyph()
        .skip(required_whitespace())
        .and(choice((
            value_record()
                .and(optional(
                    required_whitespace()
                        .with(glyph_class_or_glyph())
                        .skip(required_whitespace())
                        .and(value_record())
                ))
                .map(|(vr1, second_glyph)| {
                    match second_glyph {
                        None => Either2::A(vr1),
                        Some((second_glyph_class, vr2)) =>
                            Either2::B((second_glyph_class, (vr1, Some(vr2))))
                    }
                }),

            glyph_class_or_glyph()
                .skip(required_whitespace())
                .and(value_record())
                .map(|(second_glyph_class, vr)|
                    Either2::B((second_glyph_class, (vr, None))))
        )))

        .map(|(glyph_class, rest)| {
            match rest {
                Either2::A(value_record) => {
                    println!(" yo {:?}", &glyph_class);
                    Position::SingleAdjustment {
                        glyph_class,
                        value_record
                    }
                },

                Either2::B((second_glyph_class, value_records)) =>
                    Position::Pair {
                        glyph_classes: [glyph_class, second_glyph_class],
                        value_records
                    }
            }
        })
}

pub(crate) fn position<Input>() -> impl Parser<FeaRsStream<Input>, Output = Position>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    #[derive(Debug, Clone)]
    enum Mode {
        Normal,
        Ignore,
        Enumerate
    };

    choice((
        literal_ignore_case("ignore")
            .map(|_| Mode::Ignore)
            .skip(required_whitespace()),

        literal_ignore_case("enum")
            .skip(optional(literal_ignore_case("erate")))
            .map(|_| Mode::Enumerate)
            .skip(required_whitespace()),

        value(Mode::Normal)
    ))

        .skip(literal_ignore_case("pos"))
        .skip(optional(literal_ignore_case("ition")))
        .skip(required_whitespace())

        .and(look_ahead(take_until(space())))

        .then(|(_mode, typ): (Mode, Vec<_>)| {
            dispatch!(&*typ;
                b"base" => mark_to_base(),
                b"cursive" => cursive(),
                b"ligature" => ligature(),
                b"mark" => mark_to_mark(),
                _ => single_or_pair())
        })
}
