use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::lookup_list::*;

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct PairPosFormat1Header {
    pub format: u16,
    pub coverage_offset: u16,
    pub value_format_1: u16,
    pub value_format_2: u16,
    pub pair_set_count: u16
}

#[derive(Debug)]
pub struct PairValueRecord {
    pub second_glyph: u16,
    pub records: (ValueRecord, ValueRecord)
}

impl PairValueRecord {
    fn decode_from_be_bytes(bytes: &[u8], first_vr_size: usize, value_formats: (u16, u16)) -> Self {
        Self {
            second_glyph: decode_u16_be(bytes, 0),
            records: (
                ValueRecord::decode_from_be_bytes(&bytes[2..], value_formats.0),
                ValueRecord::decode_from_be_bytes(
                    &bytes[2 + first_vr_size..], value_formats.1))
        }
    }
}

#[derive(Debug)]
pub enum GPOSLookup {
    PairGlyphs(Vec<Vec<PairValueRecord>>)
}

#[derive(Debug)]
pub struct GPOSSubtable {
    pub coverage: Coverage,
    pub lookup: GPOSLookup,
}

impl GPOSLookup {
    #[inline]
    fn decode_pairs(bytes: &[u8]) -> Vec<Vec<PairValueRecord>> {
        let header: PairPosFormat1Header = decode_from_slice(bytes);

        let value_formats =
            (header.value_format_1, header.value_format_2);

        let vr_sizes = (
            value_formats.0.count_ones() as usize * 2,
            value_formats.1.count_ones() as usize * 2
        );

        let encoded_table_len = 2usize + vr_sizes.0 + vr_sizes.1;

        decode_from_pool(header.pair_set_count,
            &bytes[PairPosFormat1Header::PACKED_LEN..])
            .map(|offset: u16| {
                let table = &bytes[offset as usize..];
                let count = decode_u16_be(table, 0);

                (0..count)
                    .map(|i| {
                        let start = 2 + (i as usize * encoded_table_len);
                        PairValueRecord::decode_from_be_bytes(&table[start..],
                            vr_sizes.0 as usize, value_formats)
                    })
                .collect()
            })
        .collect()
    }

    #[inline]
    fn decode_from_be_bytes(bytes: &[u8], format: u16) -> Result<Self, ()> {
        Ok(match format {
            1 => Self::PairGlyphs(Self::decode_pairs(bytes)),
            _ => return Err(())
        })
    }
}

impl TTFDecode for GPOSSubtable {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> Self {
        let format = decode_u16_be(bytes, 0);
        let coverage = {
            let offset = decode_u16_be(bytes, 2) as usize;

            // FIXME: unwrap()
            Coverage::decode_from_be_bytes(&bytes[offset..]).unwrap()
        };

        // FIXME: unwrap()
        let lookup = GPOSLookup::decode_from_be_bytes(bytes, format).unwrap();

        GPOSSubtable {
            coverage,
            lookup
        }
    }
}

impl TTFEncode for GPOSSubtable {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> CompileResult<usize> {
        Ok(buf.bytes.len())
    }
}
