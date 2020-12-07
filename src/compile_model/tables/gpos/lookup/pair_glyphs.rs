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

#[allow(dead_code)]
struct PairGlyphsSplittingEncode<'a, 'buf> {
    pair_glyphs: &'a PairGlyphs,
    buf: &'buf mut EncodeBuf,

    items: iter::Peekable<btree_map::Iter<'a, u16, Vec<PairValueRecord>>>,

    value_formats: (u16, u16),
    vr_sizes: (usize, usize),

    have_encoded: bool
}

macro_rules! try_res {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Some(Err(e))
        }
    }
}

impl<'a, 'buf> Iterator for PairGlyphsSplittingEncode<'a, 'buf> {
    type Item = EncodeResult<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let value_formats = self.value_formats;
        let vr_sizes = self.vr_sizes;

        let start = self.buf.bytes.len();
        self.buf.reserve_bytes(PairPosFormat1Header::PACKED_LEN);

        let record_start = self.buf.bytes.len();

        let mut pool = EncodeBuf::new();
        pool.should_optimize_filesize = self.buf.should_optimize_filesize;

        let pair_value_record_size = u16::PACKED_LEN + vr_sizes.0 + vr_sizes.1;

        let mut items = self.items.clone();
        let mut c = pool.bytes.len();

        // using a loop {} instead of a for {} here because we need to peek at the end of the loop
        // to see if we proceed to the next iteration. for {} holds the iterator borrow for the
        // body of the loop.
        loop {
            let (_, set) = match items.next() {
                Some(x) => x,
                None => break
            };

            let pair_set_start = pool.bytes.len();

            let pair_set_count: u16 =
                try_res!(set.len().checked_into("PairSet", "pair set count"));

            pool.append(&pair_set_count).unwrap();
            pool.reserve_bytes(pair_value_record_size * set.len());

            for pair in set {
                try_res!(pool.encode_at(&pair.second_glyph, c));
                c += u16::PACKED_LEN;

                try_res!(pair.records.0.encode_to_format(&mut pool, value_formats.0, pair_set_start, c));
                c += vr_sizes.0;

                try_res!(pair.records.1.encode_to_format(&mut pool, value_formats.1, pair_set_start, c));
                c += vr_sizes.1;
            }

            try_res!(self.buf.append(&try_res!(u16::checked_from(
                    "PairGlyphs", "pair set pool offset", pair_set_start))));

            if let Some((_, next_set)) = items.peek() {
                let next_set_size = u16::PACKED_LEN + (pair_value_record_size * next_set.len());

                // with space for the offset record
                let next_fixed_size = self.buf.bytes.len() - start + u16::PACKED_LEN;
                let next_pool_size = pool.bytes.len() + next_set_size;

                if (next_fixed_size + next_pool_size) > 0xFFFE {
                    break;
                }
            }
        }

        let pool_start = try_res!(self.buf.append(&pool));

        // FIXME: update the offset records to take pool_start into account

        self.items = items;

        Some(Ok(start))
    }
}

impl<'a, 'buf> TTFSubtableEncode<'a, 'buf> for PairGlyphs {
    type Iter = PairGlyphsSplittingEncode<'a, 'buf>;

    fn ttf_subtable_encode(&'a self, buf: &'buf mut EncodeBuf) -> Self::Iter {
        // we're determining common value formats for the whole table, which *could* be suboptimal
        // if the table has mixed value record formats and we have to split it – then, we should be
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

        PairGlyphsSplittingEncode {
            pair_glyphs: self,
            buf,

            items: self.sets.iter().peekable(),

            value_formats,
            vr_sizes,

            have_encoded: false
        }
    }
}
