use std::ops;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::coverage::*;


#[derive(Debug, PartialEq, Eq)]
pub struct PairValueRecord {
    pub second_glyph: u16,
    pub records: (ValueRecord, ValueRecord)
}

pub type PairSet = Vec<PairValueRecord>;

#[derive(Debug)]
pub struct PairGlyphs {
    pub sets: CoverageLookup<PairSet>,
    pub common_value_formats: Option<(u16, u16)>
}


impl PairGlyphs {
    pub fn new() -> Self {
        Self {
            sets: CoverageLookup::new(),
            common_value_formats: None
        }
    }

    pub fn new_with_value_formats(formats: (u16, u16)) -> Self {
        Self {
            sets: CoverageLookup::new(),
            common_value_formats: Some(formats)
        }
    }

    pub fn value_formats_match(&self, other: &(u16, u16)) -> bool {
        match self.common_value_formats {
            // FIXME: should None be always-matching or never-matching?
            None => true,

            Some(vf) => {
                (vf.0 & other.0) == other.0 && (vf.1 & other.1) == other.1
            }
        }
    }
}

impl Default for PairGlyphs {
    fn default() -> Self {
        Self::new()
    }
}

impl ops::Deref for PairGlyphs {
    type Target = CoverageLookup<PairSet>;

    fn deref(&self) -> &Self::Target {
        &self.sets
    }
}

impl ops::DerefMut for PairGlyphs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sets
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

    fn encode_with_vf(&self, buf: &mut EncodeBuf, value_formats: (u16, u16), pair_set_start: usize)
            -> EncodeResult<()> {
        buf.append(&self.second_glyph)?;

        self.records.0.encode_to_format(buf, value_formats.0, pair_set_start)?;
        self.records.1.encode_to_format(buf, value_formats.1, pair_set_start)?;

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
            1 => Self::decode_pairs(bytes, coverage_bytes)
                .map(|sets| PairGlyphs {
                    sets,
                    common_value_formats: None
                }),

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

        let value_formats = match self.common_value_formats {
            Some(vf) => vf,
            None => {
                self.values()
                    .map(|records| {
                        records.iter().fold((0u16, 0u16), |vr, pair|
                            (vr.0 | pair.records.0.smallest_possible_format(),
                            vr.1 | pair.records.1.smallest_possible_format()))
                    })
                    .fold((0u16, 0u16), |vr, smallest| {
                        (vr.0 | smallest.0,
                         vr.1 | smallest.1)
                    })
            }
        };

        buf.encode_pool_with_header(
            |buf| Ok(PairPosFormat1Header {
                format: 1,
                coverage_offset: (self.sets.ttf_encode(buf)? - start) as u16,
                value_format_1: value_formats.0,
                value_format_2: value_formats.1,
                pair_set_count: self.len() as u16
            }),

            self.values(),
            |offset, _| offset,
            |buf, &set| {
                let pair_set_start = buf.bytes.len();

                buf.append(&(set.len() as u16))?;

                for pair in set {
                    pair.encode_with_vf(buf, value_formats, pair_set_start)?;
                }

                Ok(())
            })
    }
}
