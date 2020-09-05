use std::io::prelude::*;

use combine::{
    Parser,
    stream,

    eof,

    parser::repeat::many,
};

use combine::stream::{
    buffered,
    read,
    easy
};

use ascii::ToAsciiChar;

use crate::parse_model::*;


pub use combine::stream::position::SourcePosition;


/****************************************************************************
 * parser state
 ****************************************************************************/

pub(crate) struct FeaRsParserState {
    pub development_glyph_names: bool
}

pub(crate) type FeaRsStream<S> = stream::state::Stream<S, FeaRsParserState>;

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

        Ok((definitions, stream)) => {
            let _state = stream.state;
            Ok(definitions)
        }
    }
}

// helper stub function so that we're not paying the massive monomorphisation cost on every
// recompile of client code
pub fn parse_file(file: ::std::fs::File) -> Result<Vec<TopLevelStatement>, ()> {
    parse_all(file)
}
