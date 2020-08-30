use std::ops;
use std::collections::HashMap;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::coverage::*;


#[derive(Debug, Hash, PartialEq, Eq)]
pub struct PairValueRecord {
    pub second_glyph: u16,
    pub records: (ValueRecord, ValueRecord)
}

pub type PairSet = Vec<PairValueRecord>;

#[derive(Debug)]
pub struct PairGlyphs(pub CoverageLookup<PairSet>);


impl Default for PairGlyphs {
    fn default() -> Self {
        Self(CoverageLookup::new())
    }
}

impl ops::Deref for PairGlyphs {
    type Target = CoverageLookup<PairSet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for PairGlyphs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PairValueRecord {
    fn decode_with_vf(bytes: &[u8], first_vr_size: usize, value_formats: (u16, u16)) -> Self {
        Self {
            second_glyph: decode_u16_be(bytes, 0),
            records: (
                ValueRecord::decode_from_format(&bytes[2..], value_formats.0),
                ValueRecord::decode_from_format(
                    &bytes[2 + first_vr_size..], value_formats.1))
        }
    }

    fn encode_with_vf(&self, buf: &mut EncodeBuf, value_formats: (u16, u16)) -> EncodeResult<()> {
        buf.append(&self.second_glyph)?;

        self.records.0.encode_to_format(buf, value_formats.0)?;
        self.records.1.encode_to_format(buf, value_formats.1)?;

        Ok(())
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct PairPosFormat1Header {
    pub format: u16,
    pub coverage_offset: u16,
    pub value_format_1: u16,
    pub value_format_2: u16,
    pub pair_set_count: u16
}

impl PairGlyphs {
    #[inline]
    fn decode_pairs(bytes: &[u8], coverage_bytes: &[u8]) -> DecodeResult<CoverageLookup<PairSet>> {
        let header: PairPosFormat1Header = decode_from_slice(bytes);

        let value_formats =
            (header.value_format_1, header.value_format_2);

        let vr_sizes = (
            value_formats.0.count_ones() as usize * 2,
            value_formats.1.count_ones() as usize * 2
        );

        let encoded_table_len = 2usize + vr_sizes.0 + vr_sizes.1;

        let sets = decode_from_pool(header.pair_set_count,
            &bytes[PairPosFormat1Header::PACKED_LEN..])
            .map(|offset: u16| {
                let table = &bytes[offset as usize..];
                let count = decode_u16_be(table, 0);

                (0..count)
                    .map(|i| {
                        let start = 2 + (i as usize * encoded_table_len);
                        PairValueRecord::decode_with_vf(&table[start..],
                            vr_sizes.0 as usize, value_formats)
                    })
                .collect()
            });

        CoverageLookup::decode_with_lookup(coverage_bytes, sets)
    }

    #[inline]
    fn decode_from_format(bytes: &[u8], coverage_bytes: &[u8], format: u16) -> DecodeResult<Self> {
        match format {
            1 => Self::decode_pairs(bytes, coverage_bytes).map(PairGlyphs),
            _ => return Err(DecodeError::InvalidValue("format",
                    "GPOS subtable".into()))
        }
    }
}

impl TTFDecode for PairGlyphs {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let format = decode_u16_be(bytes, 0);
        let coverage_bytes = {
            let offset = decode_u16_be(bytes, 2) as usize;
            &bytes[offset..]
        };

        PairGlyphs::decode_from_format(bytes, coverage_bytes, format)
    }
}

impl TTFEncode for PairGlyphs {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let sets = &self.0;

        buf.bytes.resize(start + PairPosFormat1Header::PACKED_LEN, 0u8);

        let value_formats = sets.values()
            .map(|records| {
                records.iter().fold((0u16, 0u16), |vr, pair|
                    (vr.0 | pair.records.0.smallest_possible_format(),
                        vr.1 | pair.records.1.smallest_possible_format()))
            })
            .fold((0u16, 0u16), |vr, smallest| {
                    (vr.0 | smallest.0,
                        vr.1 | smallest.1)
            });

        let mut record_start = buf.bytes.len();
        buf.bytes.resize(record_start + (u16::PACKED_LEN * sets.len()), 0u8);

        let mut dedup = HashMap::new();

        for set in sets.values() {
            if let Some(offset) = dedup.get(&set) {
                buf.encode_at(offset, record_start)?;
                record_start += u16::PACKED_LEN;

                continue;
            }

            let offset = (buf.bytes.len() - start) as u16;
            buf.encode_at(&offset, record_start)?;
            record_start += u16::PACKED_LEN;

            dedup.insert(set, offset);

            buf.append(&(set.len() as u16))?;

            for pair in set {
                pair.encode_with_vf(buf, value_formats)?;
            }
        }

        let header = PairPosFormat1Header {
            format: 1,
            coverage_offset: (sets.ttf_encode(buf)? - start) as u16,
            value_format_1: value_formats.0,
            value_format_2: value_formats.1,
            pair_set_count: sets.len() as u16
        };

        buf.encode_at(&header, start)?;

        Ok(start)
    }
}