use std::collections::HashMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::glyph_class::*;
use crate::glyph_order::*;

use crate::compile_model::compiler_state::*;
use crate::compile_model::tables::gpos::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::coverage::*;

use crate::parse_model::MarkClassName;


#[derive(Debug, Default)]
pub struct MarkToBase {
    classes: HashMap<MarkClassName, u16>,

    marks: CoverageLookup<MarkRecord>,
    bases: CoverageLookup<HashMap<u16, Anchor>>,
}

impl MarkToBase {
    fn add_mark(&mut self, glyph_order: &GlyphOrder, mark_class: &[MarkClassGlyphClass], class_id: u16) -> Result<(), GlyphOrderError> {
        for (glyph_class, anchor) in mark_class {
            for glyph in glyph_class.iter_glyphs(glyph_order) {
                let was_present = self.marks
                    .insert(glyph?, MarkRecord {
                        class_id,
                        anchor: anchor.into()
                    })
                    .is_some();
                if was_present {
                    panic!("glyph class overlap in mark to base");
                }
            }
        }

        Ok(())
    }

    pub fn add_mark_class(&mut self, glyph_order: &GlyphOrder, base_class: &GlyphClass,
        anchor: &Anchor, name: &MarkClassName, mark_class: &[MarkClassGlyphClass]) -> Result<(), GlyphOrderError>
    {
        let class_id =
            if self.classes.contains_key(name) {
                *self.classes.get(name).unwrap()
            } else {
                let id = self.classes.len() as u16;
                self.add_mark(glyph_order, mark_class, id)?;
                self.classes.insert(name.clone(), id);
                id
            };

        for base_glyph in base_class.iter_glyphs(glyph_order) {
            self.bases.entry(base_glyph?)
                .or_default()
                .insert(class_id, anchor.clone());
        }

        Ok(())
    }
}

#[derive(Debug, PackedSize, DecodeBE, EncodeBE)]
struct MarkBasePosFormat1Header {
    format: u16,
    mark_coverage_offset: u16,
    base_coverage_offset: u16,
    mark_class_count: u16,
    mark_array_offset: u16,
    base_array_offset: u16
}

impl TTFEncode for MarkToBase {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let marks = self.marks.values();

        buf.defer_header_encode(
            move |buf| Ok(MarkBasePosFormat1Header {
                format: 1,
                mark_coverage_offset: (buf.append(&self.marks)? - start) as u16,
                base_coverage_offset: (buf.append(&self.bases)? - start) as u16,
                mark_class_count: self.classes.len() as u16,
                mark_array_offset: (marks.ttf_encode_mark_array(buf)? - start) as u16,
                base_array_offset: 0
            }),

            |_| {
                // we don't actually have any fixed items after the header for mark to base.
                // all the data is referenced by offsets in the header.

                Ok(())
            })
    }
}
