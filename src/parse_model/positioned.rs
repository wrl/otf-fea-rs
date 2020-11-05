use combine::{
    Parser,
    Stream,
    error::ParseError,
    position
};

use crate::parser::*;
use crate::{
    Positioned,
    SourceSpan
};

#[inline]
fn positions_to_span(start: SourcePosition, end: SourcePosition) -> SourceSpan {
    SourceSpan {
        start: crate::SourcePosition {
            line: start.line as usize,
            column : start.column as usize,
        },

        end: crate::SourcePosition {
            line: end.line as usize,
            column: end.column as usize,
        },
    }
}

#[inline]
pub(crate) fn positioned<Input, T, P>(p: P) -> impl Parser<Input, Output = Positioned<T>>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          P: Parser<Input, Output = T>
{
    position()
        .and(p)
        .and(position())

        .map(|((start, value), end)| Positioned {
            value,
            span: positions_to_span(start, end)
        })
}

