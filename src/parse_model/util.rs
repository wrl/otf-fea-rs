use std::{
    str,
    str::FromStr,
    convert::TryFrom,
};

use combine::{
    Parser,
    Stream,

    error::{
        ParseError
    }
};

use combine::{
    tokens_cmp,
    optional,
    token,

    attempt,
    look_ahead,
    satisfy,

    // macros
    choice
};

use combine::parser::repeat::{
    skip_until,
    many1,
    many
};

use combine::parser::byte::{
    newline,
    letter,
    space,
    digit
};

pub(crate) use crate::util::{
    Either2,
    Either3
};

use crate::parser::*;

pub use combine::stream::StreamErrorFor;
pub use combine::error::StreamError;

#[macro_export]
macro_rules! parse_bail (
    ($Input:ty, $position:ident, $exp:expr) => {
        return Err(<$Input>::Error::from_error($position,
                StreamErrorFor::<$Input>::message_format($exp)).into());
    }
);

#[inline]
pub(crate) fn literal_ignore_case<Input>(lit: &str) -> impl Parser<Input, Output = std::str::Bytes>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    tokens_cmp(lit.bytes(), |l, r| l.eq_ignore_ascii_case(&r))
}

#[inline]
pub(crate) fn literal<Input>(lit: &str) -> impl Parser<Input, Output = std::str::Bytes>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    tokens_cmp(lit.bytes(), |l, r| l.eq(&r))
}

#[inline]
pub(crate) fn comment<Input>() -> impl Parser<Input, Output = ()>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    token(b'#').map(|_| ())
        .skip(skip_until(newline()))
        .skip(newline())
}

#[inline]
pub(crate) fn optional_whitespace<Input>() -> impl Parser<Input, Output = ()>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    many(choice((
        space()
            .map(|_| ()),
        comment()
    )))
        .expected("whitespace")
}

#[inline]
pub(crate) fn required_whitespace<Input>() -> impl Parser<FeaRsStream<Input>, Output = ()>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    space()
        .and(optional_whitespace())
        .map(|_| ())
}

pub(crate) fn uinteger<Input>() -> impl Parser<FeaRsStream<Input>, Output = usize>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(many1(digit()))
        .flat_map(|(position, n): (_, Vec<u8>)| {
            // unsafe is fine here. we've verified that this vec only contains digits.
            let as_str = unsafe { str::from_utf8_unchecked(&*n) };

            usize::from_str_radix(as_str, 10)
                .map_err(|_| {
                    Input::Error::from_error(position,
                        StreamErrorFor::<Input>::expected_static_message(
                            "couldn't parse integer")).into()
                })
        })
}

pub(crate) fn hex_uint<Input>() -> impl Parser<FeaRsStream<Input>, Output = usize>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .skip(token(b'0'))
        .skip(token(b'x'))
        .and(many1(digit()))
        .flat_map(|(position, n): (_, Vec<u8>)| {
            // unsafe is fine here. we've verified that this vec only contains digits.
            let as_str = unsafe { str::from_utf8_unchecked(&*n) };

            usize::from_str_radix(as_str, 16)
                .map_err(|_| {
                    Input::Error::from_error(position,
                        StreamErrorFor::<Input>::expected_static_message(
                            "couldn't parse integer")).into()
                })
        })
}

pub(crate) fn number<Input>() -> impl Parser<FeaRsStream<Input>, Output = isize>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(optional(choice((
            token(b'-'),
            token(b'+'))
        )))
        .and(choice((
            attempt(hex_uint()),
            uinteger()
        )))

        .flat_map(|((position, sign), int)| {
            isize::try_from(int)
                .map(|val|
                    if let Some(b'-') = sign {
                        val * -1isize
                    } else {
                        val
                    })

                .map_err(|_|
                    Input::Error::from_error(position,
                        StreamErrorFor::<Input>::expected_static_message(
                            "couldn't convert integer to signed")).into())
        })
}

pub(crate) fn decimal_number<Input>() -> impl Parser<FeaRsStream<Input>, Output = f64>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    combine::position()
        .and(many1(
                satisfy(|t: u8| t.is_ascii_digit() || t == b'.' || t == b'-')
        ))
        .flat_map(|(position, digits): (_, Vec<_>)| {
            // unsafe is fine here. we've verified that this vec only contains valid characters
            let as_str = unsafe { str::from_utf8_unchecked(&*digits) };

            f64::from_str(as_str)
                .map_err(|_|
                    Input::Error::from_error(position,
                        StreamErrorFor::<Input>::expected_static_message(
                            "couldn't parse digits as f64")).into())
        })
}

#[inline]
pub(crate) fn peek<Input, P>(p: P) -> impl Parser<Input>
where
    Input: Stream,
    P: Parser<Input>
{
    attempt(look_ahead(p))
}

#[inline]
pub(crate) fn keyword<Input>() -> impl Parser<Input, Output = String>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    // from_utf8_unchecked() is safe here because letter() only matches ASCII chars.
    many1(letter()).map(|x| unsafe { String::from_utf8_unchecked(x) })
}
