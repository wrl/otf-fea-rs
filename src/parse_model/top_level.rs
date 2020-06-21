use combine::{
    Parser,
    Stream,

    look_ahead,

    token,

    parser::repeat::take_until,
    parser::byte::space,

    // macros
    dispatch
};

use combine::error::ParseError;

use crate::parser::FeaRsStream;
use super::*;

#[derive(Debug)]
pub enum TopLevelStatement {
    LanguageSystem(LanguageSystem),

    AnchorDefinition(AnchorDefinition),
    MarkClass(MarkClass),
    NamedGlyphClass(NamedGlyphClass),

    FeatureDefinition(FeatureDefinition),
    LookupDefinition(LookupDefinition),

    Anonymous(Anonymous),

    Table(Table),
}

pub(crate) fn top_level_statement<Input>() -> impl Parser<FeaRsStream<Input>, Output = TopLevelStatement>
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

                b"anon" | b"anonymous" =>
                    anonymous()
                        .map(TopLevelStatement::Anonymous),

                kwd if kwd[0] == b'@' =>
                    named_glyph_class()
                        .map(TopLevelStatement::NamedGlyphClass),

                _ => combine::unexpected_any("token"))
        })
        .skip(optional_whitespace())
        .skip(token(b';'))
}
