use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::tables::gpos::*;
use crate::compile_model::coverage::*;


#[derive(Debug)]
pub struct Anchors {
    entry: Anchor,
    exit: Anchor
}

#[derive(Debug, Default)]
pub struct Cursive(pub CoverageLookup<Anchors>);

impl Cursive {
    pub fn add_rule(&mut self, glyph_id: u16, entry: Anchor, exit: Anchor) {
        self.0.insert(glyph_id, Anchors {
            entry,
            exit
        });
    }
}

#[derive(PackedSize, DecodeBE, EncodeBE)]
struct CursivePosFormat1Header {
    format: u16,
    coverage_offset: u16,
    entry_exit_count: u16
}

#[derive(PackedSize, DecodeBE, EncodeBE)]
struct EntryExitRecord {
    entry_anchor_offset: u16,
    exit_anchor_offset: u16
}

impl TTFEncode for Cursive {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.encode_pool_2_with_header(
            |buf| Ok(CursivePosFormat1Header {
                format: 1,
                coverage_offset: (self.0.ttf_encode(buf)? - start) as u16,
                entry_exit_count: self.0.len() as u16
            }),

            self.0.values(),

            // FIXME: we're encoding two discrete items into the pool here, but the EncodeBuf pool
            // funcs assume one pool item per record, where here we have 2.
            //
            // as a matter of fact, this doesn't work at all, so.
            // need to refactor the pool encode funcs.
            |(entry_anchor_offset, exit_anchor_offset), _| EntryExitRecord {
                entry_anchor_offset,
                exit_anchor_offset
            },

            |buf, anchors| {
                let entry = if anchors.entry.should_encode(buf) {
                    buf.append(&anchors.entry)? - start
                } else {
                    0
                };

                let exit = if anchors.exit.should_encode(buf) {
                    buf.append(&anchors.exit)? - start
                } else {
                    0
                };

                Ok((
                    entry as u16,
                    exit as u16
                ))
            })
    }
}
