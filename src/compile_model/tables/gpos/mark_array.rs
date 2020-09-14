use endian_codec::{EncodeBE, DecodeBE, PackedSize};

use crate::compile_model::util::encode::*;
use super::Anchor;


#[derive(Debug)]
pub struct MarkRecord {
    pub class_id: u16,
    pub anchor: Anchor
}

pub trait MarkArrayTTFEncode {
    fn ttf_encode_mark_array(self, buf: &mut EncodeBuf) -> EncodeResult<usize>;
}

#[derive(EncodeBE, DecodeBE, PackedSize)]
struct MarkRecordEncoded {
    class_id: u16,
    anchor_offset: u16
}

// if this is an implementation of TTFEncode, rustc thinks that there is an overlap/conflict with
// the blanket <T: EncodeBE> impl in util::encode, unhelpfully telling us that a downstream crate
// could implement EncodeBE for `&'a [&'a MarkRecord]`.
//
// this is complete bullshit since Anchor has variable-length encoding and hence cannot have
// EncodeBE implemented for it, but there is no way to communicate this to rustc since negative
// trait bounds (or any other way to communicate "this trait cannot be implemented for this type")
// are unsupported.
//
// so, we're stuck with this sub-optimal bespoke trait just used here.
impl<'a, I> MarkArrayTTFEncode for I
    where I: Iterator<Item = &'a MarkRecord> + ExactSizeIterator
{
    fn ttf_encode_mark_array(self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.append(&(self.len() as u16))?;

        // FIXME: should dedup on the anchor
        buf.encode_pool(start, self,
            |anchor_offset, record| MarkRecordEncoded {
                class_id: record.class_id,
                anchor_offset
            },
            |buf, record| buf.append(&record.anchor))?;

        Ok(start)
    }
}
