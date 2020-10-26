use std::convert::TryFrom;

use crate::compile_model::util::encode::*;
use crate::compile_model::error::*;

use crate::parse_model as pm;


#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Device {
    pub start_size: u16,
    pub end_size: u16,
}

impl TryFrom<&pm::Device> for Device {
    type Error = CompileError;

    fn try_from(_parsed: &pm::Device) -> CompileResult<Self> {
        Ok(Device {
            start_size: 0,
            end_size: 0,
        })
    }
}

impl TTFEncode for Device {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        Ok(start)
   }
}
