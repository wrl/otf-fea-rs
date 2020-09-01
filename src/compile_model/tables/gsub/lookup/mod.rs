use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;


#[derive(Debug)]
pub enum GSUBLookup { }

impl TTFDecode for GSUBLookup {
    fn ttf_decode(_bytes: &[u8]) -> DecodeResult<Self> {
        // let lookup_type = decode_u16_be(bytes, 0);

        panic!("unimplemented");
    }
}

impl TTFEncode for GSUBLookup {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let _start = buf.bytes.len();

        panic!("unimplemented");
    }
}
