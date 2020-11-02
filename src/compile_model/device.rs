use std::convert::TryFrom;
use std::collections::BTreeMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::encode::*;
use crate::compile_model::error::*;

use crate::parse_model as pm;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Device {
    // ppem_size => pixel_adjustment
    pub adjustments: BTreeMap<u16, i8>
}

impl TryFrom<&pm::Device> for Device {
    type Error = CompileError;

    fn try_from(parsed: &pm::Device) -> CompileResult<Self> {
        let mut adjustments = BTreeMap::new();

        if let pm::Device::Adjustments(ref parsed_adj) = parsed {
            for adj in parsed_adj {
                let ppem_size = u16::try_from(adj.ppem_size)
                    .map_err(|_| CompileError::Overflow {
                        ty: "u16",
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

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum DeltaFormat {
    TwoBit   = 0x0001,
    FourBit  = 0x0002,
    EightBit = 0x0003,
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct DeviceHeader {
    pub start_ppem: u16,
    pub end_ppem: u16,
    pub delta_format: u16
}

macro_rules! pack_ppem_base {
    ($self:ident, $range:ident, $done:ident, $bits:expr) => {
        match $range.next() {
            Some(ppem) => match $self.adjustments.get(&ppem) {
                Some(adj) => {
                    let adj = *adj as u8;
                    let sign = ((adj & (1 << 7)) != 0) as u16;
                    let mask = (1 << ($bits - 1)) - 1;

                    ((adj as u16) & mask) | (sign << ($bits - 1))
                },
                None => 0u16
            },
            None => {
                $done = true;
                0u16
            }
        }
    }
}

impl Device {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.adjustments.is_empty()
    }

    fn pack_eights(&self, start_ppem: u16, end_ppem: u16, buf: &mut EncodeBuf) -> EncodeResult<()> {
        let mut range = start_ppem ..= end_ppem;
        let mut done = false;

        macro_rules! pack_ppem {
            () => {
                pack_ppem_base!(self, range, done, 8)
            }
        }

        while !done {
            let packed: u16 =
                 (pack_ppem!() << 8)
                | pack_ppem!();

            buf.append(&packed)?;
        }

        Ok(())
    }

    fn pack_fours(&self, start_ppem: u16, end_ppem: u16, buf: &mut EncodeBuf) -> EncodeResult<()> {
        let mut range = start_ppem ..= end_ppem;
        let mut done = false;

        macro_rules! pack_ppem {
            () => {
                pack_ppem_base!(self, range, done, 4)
            }
        }

        while !done {
            let packed: u16 =
                  (pack_ppem!() << 12)
                | (pack_ppem!() << 8)
                | (pack_ppem!() << 4)
                |  pack_ppem!();

            buf.append(&packed)?;
        }

        Ok(())
    }

    fn pack_twos(&self, start_ppem: u16, end_ppem: u16, buf: &mut EncodeBuf) -> EncodeResult<()> {
        let mut range = start_ppem ..= end_ppem;
        let mut done = false;

        macro_rules! pack_ppem {
            () => {
                pack_ppem_base!(self, range, done, 2)
            }
        }

        while !done {
            let packed: u16 =
                  (pack_ppem!() << 14)
                | (pack_ppem!() << 12)
                | (pack_ppem!() << 10)
                | (pack_ppem!() << 8)
                | (pack_ppem!() << 6)
                | (pack_ppem!() << 4)
                | (pack_ppem!() << 2)
                |  pack_ppem!();

            buf.append(&packed)?;
        }

        Ok(())
    }
}

impl TTFEncode for Device {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let mut format = DeltaFormat::TwoBit;

        if self.adjustments.len() == 0 {
            // FIXME: what do we do here? encode an empty device table, or pass some information up
            // to the caller?

            buf.append(&DeviceHeader {
                start_ppem: 0,
                end_ppem: 0,
                delta_format: DeltaFormat::EightBit as u16
            })?;
            buf.append(&0u16)?;
        }

        let start_ppem = *self.adjustments.keys().next().unwrap();
        let end_ppem = *self.adjustments.keys().last().unwrap();

        for v in self.adjustments.values() {
            let v = *v;

            if v > 7 || v < -8 {
                format = DeltaFormat::EightBit;
            } else if v > 1 || v < -2 {
                format = DeltaFormat::FourBit;
            }
        }

        buf.append(&DeviceHeader {
            start_ppem,
            end_ppem,
            delta_format: format as u16
        })?;

        match format {
            DeltaFormat::TwoBit => self.pack_twos(start_ppem, end_ppem, buf)?,
            DeltaFormat::FourBit => self.pack_fours(start_ppem, end_ppem, buf)?,
            DeltaFormat::EightBit => self.pack_eights(start_ppem, end_ppem, buf)?,
        };

        Ok(start)
   }
}
