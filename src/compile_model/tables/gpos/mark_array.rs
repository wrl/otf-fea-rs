use std::ops;

use endian_codec::{EncodeBE, DecodeBE, PackedSize};

use crate::compile_model::util::encode::*;
use super::Anchor;


#[derive(Debug)]
pub struct MarkRecord(pub u16, pub Anchor);

pub trait MarkArrayTTFEncode {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize>;
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
impl<'a, T> MarkArrayTTFEncode for T
    where T: ops::Deref<Target = [&'a MarkRecord]>
{
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

        buf.append(&(self.len() as u16))?;

        // FIXME: should dedup on the anchor
        buf.encode_pool(start, self.iter(),
            |anchor_offset, record| MarkRecordEncoded {
                class_id: record.0,
                anchor_offset
            },
            |buf, record| buf.append(&record.1))?;

        Ok(start)
    }
}
