use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;
use crate::compile_model::value_record::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::util;
use crate::compile_model::TTFTable;

#[derive(Debug)]
pub struct GPOS {
    script_list: ScriptList,
    feature_list: FeatureList,
    lookup_list: LookupList<GPOSLookup>,
    feature_variations_offset: Option<u16>
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct PairPosFormat1Header {
    pub value_format_1: u16,
    pub value_format_2: u16,
    pub pair_set_count: u16
}

#[derive(Debug)]
struct PairValueRecord {
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
struct GPOSLookup {
    pub format: u16,
    pub coverage: Coverage,
    pub pairs: Vec<Vec<PairValueRecord>>
}

impl GPOSLookup {
    #[inline]
    fn decode_pairs(bytes: &[u8], header: PairPosFormat1Header) -> Vec<Vec<PairValueRecord>> {
        let value_formats =
            (header.value_format_1, header.value_format_2);

        let vr_sizes = (
            value_formats.0.count_ones() * 2,
            value_formats.1.count_ones() * 2
        );

        let encoded_table_len = util::align_len(
            (2 + vr_sizes.0 + vr_sizes.1) as usize);

        decode_from_pool(header.pair_set_count,
            &bytes[4 + PairPosFormat1Header::PACKED_LEN..])
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
}

impl TTFTable for GPOSLookup {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let format = decode_u16_be(bytes, 0);
        let coverage = {
            let offset = decode_u16_be(bytes, 2) as usize;
            Coverage::decode_from_be_bytes(&bytes[offset..])
                .ok_or(())?
        };

        let header: PairPosFormat1Header = decode_from_slice(&bytes[4..]);
        let pairs = Self::decode_pairs(bytes, header);

        Ok(GPOSLookup {
            format,
            coverage,
            pairs
        })
    }
}

fn decode_from_be_bytes<T>(bytes: &[u8]) -> T
    where T: DecodeBE
{
    T::decode_from_be_bytes(&bytes[..T::PACKED_LEN])
}

impl GPOS {
    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let version: Version = decode_from_be_bytes(bytes);

        let offsets: Offsets = match (version.major, version.minor) {
            (1, 0) => Header_1_0::decode_from_be_bytes(bytes).into(),
            (1, 1) => Header_1_1::decode_from_be_bytes(bytes).into(),

            _ => return Err(())
        };

        Ok(GPOS {
            script_list: ScriptList::decode_from_be_bytes(&bytes[offsets.script as usize..]),
            feature_list: FeatureList::decode_from_be_bytes(&bytes[offsets.feature as usize..]),
            lookup_list: LookupList::decode_from_be_bytes(&bytes[offsets.lookup as usize..]),
            feature_variations_offset: offsets.feature_variations
        })
    }
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Version {
    major: u16,
    minor: u16
}

struct Offsets {
    script: u16,
    feature: u16,
    lookup: u16,
    feature_variations: Option<u16>
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_0 {
    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16
}

impl From<Header_1_0> for Offsets {
    fn from(header: Header_1_0) -> Self {
        Self {
            script: header.script_list_offset,
            feature: header.feature_list_offset,
            lookup: header.lookup_list_offset,
            feature_variations: None
        }
    }
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_1 {
    major: u16,
    minor: u16,
    script_list_offset: u16,
    feature_list_offset: u16,
    lookup_list_offset: u16,
    feature_variations_offset: u16
}

impl From<Header_1_1> for Offsets {
    fn from(header: Header_1_1) -> Self {
        Self {
            script: header.script_list_offset,
            feature: header.feature_list_offset,
            lookup: header.lookup_list_offset,
            feature_variations: Some(header.feature_variations_offset)
        }
    }
}
