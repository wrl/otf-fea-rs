use combine::{
    Parser,
    Stream,
    parser,

    token,
    choice,
    any,

    parser::byte::hex_digit,
    error::ParseError
};

use encoding_rs::{
    MACINTOSH,
    UTF_16BE
};

use combine::stream::StreamErrorFor;
use combine::error::StreamError;

use crate::parse_model::util::*;

#[inline]
fn string_escaped<Input, Escaped, UP, UPF, Unescape>
    (unescape_parser: UPF, unescape: Unescape)
        -> impl Parser<Input, Output = String>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          UPF: Fn() -> UP,
          UP: Parser<Input, Output = Escaped>,
          Unescape: 'static + Fn(Escaped) -> Vec<u8>
{
    combine::position()
        .skip(token(b'"'))

        .and(parser(move |input| {
            let mut parse_iter =
                choice((
                    token(b'"').map(Either3::A),
                    unescape_parser()
                        .map(Either3::B),
                    any().map(Either3::C)
                ))
                .iter(input);

            let mut res = Vec::new();

            for c in &mut parse_iter {
                match c {
                    Either3::A(_) => break,
                    Either3::B(escaped) => res.extend(unescape(escaped)),
                    Either3::C(ch) => res.push(ch)
                }
            }

            parse_iter.into_result(res)
        }))

        .flat_map(|(position, raw): (_, Vec<_>)| {
            match String::from_utf8(raw) {
                Ok(s) => Ok(s),
                Err(_) => crate::parse_bail!(Input, position,
                    "invalid UTF-8")
            }
        })
}

#[inline]
fn decode_hex(x: u8) -> u8 {
    match x {
        b'0' ..= b'9' => x - b'0',
        b'a' ..= b'f' => x - b'a' + 10,
        _ => unreachable!()
    }
}

////
// mac
////

#[inline]
fn unescape_mac((a, b): (u8, u8)) -> Vec<u8> {
    let val = (decode_hex(a) << 4) | decode_hex(b);

    // FIXME: other mac encodings?
    MACINTOSH.decode_without_bom_handling(&[val]).0.into_owned().into_bytes()
}

fn mac_escape_sequence<Input>() -> impl Parser<Input, Output = (u8, u8)>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    token(b'\\')
        .with(hex_digit().and(hex_digit()))
}

pub(crate) fn string_mac_escaped<Input>() -> impl Parser<Input, Output = String>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    string_escaped(mac_escape_sequence, unescape_mac)
}

////
// windows
////

#[inline]
fn unescape_win((a, b, c, d): (u8, u8, u8, u8)) -> Vec<u8> {
    let val = [
        (decode_hex(a) << 4) | decode_hex(b),
        (decode_hex(c) << 4) | decode_hex(d)
    ];

    UTF_16BE.decode_without_bom_handling(&val).0.into_owned().into_bytes()
}

fn win_escape_sequence<Input>() -> impl Parser<Input, Output = (u8, u8, u8, u8)>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    token(b'\\')
        .with(hex_digit()
            .and(hex_digit())
            .and(hex_digit())
            .and(hex_digit()))
        .map(|(((a, b), c), d)| (a, b, c, d))
}

pub(crate) fn string_win_escaped<Input>() -> impl Parser<Input, Output = String>
    where Input: Stream<Token = u8>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    string_escaped(win_escape_sequence, unescape_win)
}
