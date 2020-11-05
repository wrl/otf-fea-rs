use combine::{
    Parser,
    Stream,
    error::ParseError,

    parser::repeat::skip_many1,

    optional,
    parser,
    choice
};

use crate::parser::*;
use super::class_name::*;
use super::util::*;

#[derive(Debug)]
pub struct LookupFlag {
    right_to_left: bool,
    ignore_base_glyphs: bool,
    ignore_ligatures: bool,
    ignore_marks: bool,
    mark_attachment_type: Option<ClassName>,
    use_mark_filtering_set: Option<ClassName>
}

impl LookupFlag {
    fn new() -> Self {
        Self {
            right_to_left: false,
            ignore_base_glyphs: false,
            ignore_ligatures: false,
            ignore_marks: false,

            mark_attachment_type: None,
            use_mark_filtering_set: None
        }
    }
}

pub(crate) fn lookup_flag<Input>() -> impl Parser<FeaRsStream<Input>, Output = LookupFlag>
    where Input: Stream<Token = u8, Position = SourcePosition>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    literal_ignore_case("lookupflag")
        .skip(required_whitespace())
        .with(choice((
            uinteger()
                .map(|n| {
                    LookupFlag {
                        right_to_left: (n & 1) != 0,
                        ignore_base_glyphs: (n & 2) != 0,
                        ignore_ligatures: (n & 4) != 0,
                        ignore_marks: (n & 8) != 0,

                        mark_attachment_type: None,
                        use_mark_filtering_set: None
                    }
                }),

            parser(|input| {
                let mut flags = LookupFlag::new();

                let mut fmap = [
                    ("RightToLeft", Either2::A(&mut flags.right_to_left)),
                    ("IgnoreBaseGlyphs", Either2::A(&mut flags.ignore_base_glyphs)),
                    ("IgnoreLigatures", Either2::A(&mut flags.ignore_ligatures)),
                    ("IgnoreMarks", Either2::A(&mut flags.ignore_marks)),
                    ("MarkAttachmentType", Either2::B(&mut flags.mark_attachment_type)),
                    ("UseMarkFilteringSet", Either2::B(&mut flags.use_mark_filtering_set))
                ];

                skip_many1(
                    combine::position()
                    .and(keyword())
                        .skip(optional_whitespace())
                        .and(optional(class_name()))
                        .flat_map(|((position, kwd), class_name)| {
                            let ptr =
                                match fmap.iter_mut().find(|(iden, _)| iden == &kwd) {
                                    Some((_, ptr)) => ptr,
                                    None => crate::parse_bail!(Input, position, "unexpected keyword")
                                };

                            match ptr {
                                &mut Either2::A(ref mut bool_ref) if **bool_ref =>
                                        crate::parse_bail!(Input, position, "duplicate flag"),
                                &mut Either2::A(ref mut bool_ref) =>
                                    **bool_ref = true,

                                &mut Either2::B(ref mut cn_ref) if cn_ref.is_some() =>
                                        crate::parse_bail!(Input, position, "duplicate flag"),
                                &mut Either2::B(_) if class_name.is_none() =>
                                        crate::parse_bail!(Input, position, "expected class name"),
                                &mut Either2::B(ref mut cn_ref) =>
                                    **cn_ref = class_name,
                            }

                            Ok(())
                        })
                )
                    .parse_stream(input)
                    .into_result()
                    .map(|(_, commit)| (flags, commit))
            })
        )))
}
