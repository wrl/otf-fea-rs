use std::convert::TryFrom;
use std::collections::BTreeMap;

use crate::compile_model::util::encode::*;
use crate::compile_model::error::*;

use crate::parse_model as pm;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Device {
    // ppem_size => pixel_adjustment
    pub adjustments: BTreeMap<i8, i8>
}

impl TryFrom<&pm::Device> for Device {
    type Error = CompileError;

    fn try_from(parsed: &pm::Device) -> CompileResult<Self> {
        let mut adjustments = BTreeMap::new();

        if let pm::Device::Adjustments(ref parsed_adj) = parsed {
            for adj in parsed_adj {
                let ppem_size = i8::try_from(adj.ppem_size)
                    .map_err(|_| CompileError::Overflow {
                        ty: "i8",
                        scope: "Device".into(),
                        item: "ppem_size",
                        value: adj.ppem_size as usize
                    })?;

                let pixel_adjustment = i8::try_from(adj.pixel_adjustment)
                    .map_err(|_| CompileError::Overflow {
                        ty: "i8",
                        scope: "Device".into(),
                        item: "pixel_adjustment",
                        value: adj.pixel_adjustment as usize
                    })?;

                adjustments.insert(ppem_size, pixel_adjustment);
            }
        }

        Ok(Device {
            adjustments
        })
    }
}

impl TTFEncode for Device {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        Ok(start)
   }
}
