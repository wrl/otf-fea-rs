use std::ops;
use std::iter;
use std::collections::btree_map;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::value_record::*;
use crate::compile_model::coverage::*;
use crate::compile_model::lookup::*;
use crate::compile_model::util::*;


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

pub struct PairGlyphsSplittingEncoder<'a> {
    items: iter::Peekable<btree_map::Iter<'a, u16, Vec<PairValueRecord>>>,

    value_formats: (u16, u16),
    vr_sizes: (usize, usize),
}

macro_rules! try_res {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Some(Err(e))
        }
    }
}

impl<'a> TTFSubtableEncoder<'a> for PairGlyphsSplittingEncoder<'a> {
    fn encode_next_subtable(&mut self, buf: &mut EncodeBuf) -> Option<EncodeResult<usize>> {
        if self.items.len() == 0 {
            return None
        }

        let value_formats = self.value_formats;
        let vr_sizes = self.vr_sizes;

        let start = buf.bytes.len();
        buf.reserve_bytes(PairPosFormat1Header::PACKED_LEN);

        let record_start = buf.bytes.len();

        let mut pool = EncodeBuf::new();
        pool.should_optimize_filesize = buf.should_optimize_filesize;

        let pair_value_record_size = u16::PACKED_LEN + vr_sizes.0 + vr_sizes.1;

        let mut offsets: Vec<usize> = Vec::with_capacity(64);
        let mut set_count = 0usize;
        let mut c = pool.bytes.len();

        let items_clone_for_coverage = self.items.clone()
            .map(|(glyph_id, _)| *glyph_id);

        // using a loop {} instead of a for {} here because we need to peek at the end of the loop
        // to see if we proceed to the next iteration. for {} holds the iterator borrow for the
        // body of the loop.
        while let Some((_, set)) = self.items.next() {
            let pair_set_start = pool.bytes.len();

            let pair_set_count: u16 =
                try_res!(set.len().checked_into("PairSet", "pair set count"));

            try_res!(pool.append(&pair_set_count));
            pool.reserve_bytes(pair_value_record_size * set.len());

            for pair in set {
                try_res!(pool.encode_at(&pair.second_glyph, c));
                c += u16::PACKED_LEN;

                try_res!(pair.records.0.encode_to_format(&mut pool, value_formats.0, pair_set_start, c));
                c += vr_sizes.0;

                try_res!(pair.records.1.encode_to_format(&mut pool, value_formats.1, pair_set_start, c));
                c += vr_sizes.1;
            }

            offsets.push(pair_set_start);
            try_res!(buf.append(&0u16));

            set_count += 1;

            if let Some((_, next_set)) = self.items.peek() {
                let next_set_size = u16::PACKED_LEN + (pair_value_record_size * next_set.len());

                // with space for the offset record
                let next_fixed_size = buf.bytes.len() - start + u16::PACKED_LEN;
                let next_pool_size = pool.bytes.len() + next_set_size;

                if (next_fixed_size + next_pool_size) > 0xFFFE {
                    break;
                }
            }
        }

        let pool_start = try_res!(buf.append(&pool));

        for (i, offset) in offsets.into_iter().enumerate() {
            let offset = try_res!(
                u16::checked_from("PairGlyphs", "pair set pool offset", offset + pool_start));

            try_res!(buf.encode_at(&offset, record_start + (i * u16::PACKED_LEN)));
        }

        let coverage_offset: usize =
            try_res!(CoverageLookup::<()>::encode(items_clone_for_coverage.take(set_count), buf))
            - start;

        let header = PairPosFormat1Header {
            format: 1,
            coverage_offset: try_res!(coverage_offset.checked_into("PairGlyphs", "coverage_offset")),
            value_format_1: self.value_formats.0,
            value_format_2: self.value_formats.1,
            pair_set_count: try_res!(set_count.checked_into("PairGlyphs", "pair_set_count"))
        };

        try_res!(buf.encode_at(&header, start));

        Some(Ok(start))
    }
}

impl<'a> TTFSubtableEncode<'a> for PairGlyphs {
    type Encoder = PairGlyphsSplittingEncoder<'a>;

    fn ttf_subtable_encoder(&'a self) -> Self::Encoder {
        // we're determining common value formats for the whole table, which *could* be suboptimal
        // if the table has mixed value record formats and we have to split it - then, we should be
        // calculating value formats for each split table for size savings. unfortunately, we don't
        // know how many pair sets fit into each table until we encode, which requires us to have
        // already determined value formats for encoding...
        //
        // could probably solve this iteratively (i.e. calculate formats, encode, reset, calculate
        // formats, encode, etc, until the table size no longer changes), but it's almost certainly
        // not worth it for the increase in code complexity.

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

        let vr_sizes = (
            ValueRecord::size_for_format(value_formats.0),
            ValueRecord::size_for_format(value_formats.1),
        );

        PairGlyphsSplittingEncoder {
            items: self.sets.iter().peekable(),

            value_formats,
            vr_sizes,
        }
    }
}
