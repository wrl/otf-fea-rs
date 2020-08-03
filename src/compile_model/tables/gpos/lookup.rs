use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::coverage::*;

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

pub type PairSet = Vec<PairValueRecord>;

#[derive(Debug)]
pub enum GPOSLookup {
    PairGlyphs(CoverageLookup<PairSet>)
}

#[derive(Debug)]
pub struct GPOSSubtable {
    pub lookup: GPOSLookup,
}

impl GPOSLookup {
    #[inline]
    fn decode_pairs(bytes: &[u8], coverage: Coverage) -> CoverageLookup<PairSet> {
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

                println!("{}", offset);

                (0..count)
                    .map(|i| {
                        let start = 2 + (i as usize * encoded_table_len);
                        PairValueRecord::decode_with_vf(&table[start..],
                            vr_sizes.0 as usize, value_formats)
                    })
                .collect()
            });

        match coverage {
            Coverage::Glyphs(glyphs) => {
                CoverageLookup::Glyphs(
                    glyphs.into_iter()
                        .zip(sets)
                        .collect()
                )
            },

            Coverage::GlyphRanges(ranges) => {
                CoverageLookup::GlyphRanges(
                    ranges.into_iter()
                        .zip(sets)
                        .collect()
                )
            }
        }
    }

    #[inline]
    fn encode_pair_glyphs(sets: &CoverageLookup<PairSet>, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();

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

        for set in sets.values() {
            let offset = (buf.bytes.len() - start) as u16;
            buf.encode_at(&offset, record_start)?;
            record_start += u16::PACKED_LEN;

            buf.append(&(set.len() as u16))?;

            for pair in set {
                pair.encode_with_vf(buf, value_formats)?;
            }
        }

        let header = PairPosFormat1Header {
            format: 1,
            coverage_offset: 42440,
            value_format_1: value_formats.0,
            value_format_2: value_formats.1,
            pair_set_count: sets.len() as u16
        };

        buf.encode_at(&header, start)?;

        Ok(start)
    }

    #[inline]
    fn decode_from_format(bytes: &[u8], coverage: Coverage, format: u16) -> DecodeResult<Self> {
        Ok(match format {
            1 => Self::PairGlyphs(Self::decode_pairs(bytes, coverage)),
            _ => return Err(DecodeError::InvalidValue("format",
                    "GPOS subtable".into()))
        })
    }

    #[inline]
    fn encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        match self {
            Self::PairGlyphs(sets) => Self::encode_pair_glyphs(sets, buf)
        }
    }
}

impl TTFDecode for GPOSSubtable {
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        // FIXME: need to pass through lookup_type

        let format = decode_u16_be(bytes, 0);
        let coverage = {
            let offset = decode_u16_be(bytes, 2) as usize;
            Coverage::ttf_decode(&bytes[offset..])?
        };

        let lookup = GPOSLookup::decode_from_format(bytes, coverage, format)?;

        Ok(GPOSSubtable {
            lookup
        })
    }
}

impl TTFEncode for GPOSSubtable {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        self.lookup.encode(buf)
    }
}
