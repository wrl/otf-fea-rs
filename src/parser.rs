use std::io::prelude::*;

use combine::{
    Parser,
    Stream,
    stream,

    look_ahead,

    token,
    eof,

    parser::repeat::{
        take_until,
        many
    },
    parser::byte::space,

    // macros
    dispatch
};

use combine::error::ParseError;

use combine::stream::{
    buffered,
    read,
    easy,

    position::SourcePosition,
};

use ascii::ToAsciiChar;

use crate::parse_model::*;

/****************************************************************************
 * parser state
 ****************************************************************************/

pub(crate) struct FeaRsParserState {
    pub development_glyph_names: bool
}

pub(crate) type FeaRsStream<S> = stream::state::Stream<S, FeaRsParserState>;

/****************************************************************************
 * top-level statements
 ****************************************************************************/

#[derive(Debug)]
pub enum TopLevelStatement {
    LanguageSystem(LanguageSystem),

    AnchorDefinition(AnchorDefinition),
    MarkClass(MarkClass),
    NamedGlyphClass(NamedGlyphClass),

    FeatureDefinition(FeatureDefinition),
    LookupDefinition(LookupDefinition),

    Table(Table)
}

fn top_level_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TopLevelStatement>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    look_ahead(take_until(space()))
        .then(|kwd: Vec<_>| {
            dispatch!(&*kwd;
                b"table" =>
                    table()
                        .map(TopLevelStatement::Table),

                b"lookup" =>
                    lookup_definition()
                        .map(TopLevelStatement::LookupDefinition),

                b"languagesystem" =>
                    language_system()
                        .map(TopLevelStatement::LanguageSystem),

                b"feature" =>
                    feature_definition()
                        .map(TopLevelStatement::FeatureDefinition),

                b"markClass" =>
                    mark_class()
                        .map(TopLevelStatement::MarkClass),

                b"anchorDef" =>
                    anchor_definition()
                        .map(TopLevelStatement::AnchorDefinition),

                kwd if kwd[0] == b'@' =>
                    named_glyph_class()
                        .map(TopLevelStatement::NamedGlyphClass),

                _ => combine::unexpected_any("token"))
        })
        .skip(optional_whitespace())
        .skip(token(b';'))
}

/****************************************************************************
 * parse func
 ****************************************************************************/

fn format_errors<T, R>(errors: combine::easy::Errors<T, R, SourcePosition>)
where
    T: std::fmt::Debug + ToAsciiChar,
    R: std::fmt::Debug
{
    use combine::stream::easy::Info;

    macro_rules! println_with_info {
        ($fmt: expr, $info: expr) => {

            match $info {
                Info::Token(t) => eprintln!(concat!($fmt, " token {:?}"),
                    t.to_ascii_char().unwrap()),
                Info::Range(r) => eprintln!(concat!($fmt, " range {:?}"), r),
                Info::Owned(s) => eprintln!(concat!($fmt, " {}"), s),
                Info::Static(s) => eprintln!(concat!($fmt, " {}"), s)
            }
        }
    }

    eprintln!("parse error at {}:{}", errors.position.line, errors.position.column);

    for e in errors.errors {
        use combine::stream::easy::Error::*;

        match e {
            Unexpected(info) => println_with_info!("    unexpected", info),
            Expected(info) => println_with_info!("    expected", info),
            Message(info) => println_with_info!("   ", info),
            _ => println!("    {:?}", e)
        }
    }
}

pub fn parse_all<R: Read>(input: R) -> Result<Vec<TopLevelStatement>, ()> {
    println!();

    let mut parser = optional_whitespace()
        .with(many(
            top_level_statement()
                .skip(optional_whitespace())
        ))
        .skip(eof());

    let stream = FeaRsStream {
        stream:
            easy::Stream::from(
                buffered::Stream::new(
                    stream::position::Stream::with_positioner(
                        read::Stream::new(input),
                        SourcePosition::new()),
                    512)),

        state: FeaRsParserState {
            development_glyph_names: false
        }
    };

    match parser.parse(stream) {
        Err(errs) => {
            format_errors(errs);
            Err(())
        },

        Ok((definitions, _stream)) => Ok(definitions),
    }
}
