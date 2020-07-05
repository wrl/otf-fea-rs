#![allow(non_camel_case_types)]

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::parse_model::Tag;

pub mod tables;

mod script_list;
pub use script_list::{
    ScriptList,
    Script,
};

mod feature_list;
pub use feature_list::{
    FeatureList,
    FeatureRecord,
};

mod lookup_list;
pub use lookup_list::{
    LookupList,
    Lookup
};

pub mod value_record;

#[macro_use]
pub mod util;
pub use util::TTFVersion;


#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
pub struct TTFOffsetTable {
    pub version: TTFVersion,

    pub num_tables: u16,

    // (maximum power of 2 <= num_tables) * 16
    pub search_range: u16,

    // log2(maximum power of 2 <= num_tables)
    pub entry_selector: u16,

    // (num_tables * 16) - search_range
    pub range_shift: u16
}

impl TTFOffsetTable {
    pub fn new(version: TTFVersion, num_tables: u16) -> Self {
        let max_power_of_two = 15u16.saturating_sub(
            num_tables.leading_zeros() as u16);

        let search_range = (1 << max_power_of_two) * 16;
        let entry_selector = max_power_of_two;
        let range_shift = (num_tables * 16).saturating_sub(search_range);

        Self {
            version,
            num_tables,
            search_range,
            entry_selector,
            range_shift
        }
    }
}


#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
pub struct TTFTableRecord {
    pub tag: Tag,
    pub checksum: u32,
    pub offset_from_start_of_file: u32,
    pub length: u32
}

impl TTFTableRecord {
    pub fn table_data<'a>(&self, whole_file: &'a [u8]) -> &'a [u8] {
        let (start, end) = (
            self.offset_from_start_of_file as usize,
            (self.offset_from_start_of_file + self.length) as usize
        );

        &whole_file[start..end]
    }
}

pub struct EncodeBuf {
    pub(crate) bytes: Vec<u8>
}

impl EncodeBuf {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new()
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &*self.bytes
    }

    #[inline]
    pub(crate) fn append<T: TTFEncode>(&mut self, val: &T) -> Result<usize, ()> {
        // FIXME: unwrap()
        val.ttf_encode(self)
    }

    #[inline]
    pub(crate) fn encode_at<T: EncodeBE>(&mut self, val: &T, start: usize) -> Result<usize, ()> {
        let end = start + T::PACKED_LEN;

        if end > self.bytes.len() {
            return Err(())
        }

        val.encode_as_be_bytes(&mut self.bytes[start..end]);

        Ok(start)
    }
}

pub trait TTFTable: Sized {
    fn decode_from_be_bytes(bytes: &[u8]) -> Result<Self, ()>;
    fn encode_as_be_bytes(&self, buf: &mut Vec<u8>) -> Result<(), ()>;
}

pub trait TTFDecode: Sized {
    fn ttf_decode(bytes: &[u8], tag: Option<Tag>) -> Self;
}

pub trait TTFEncode: Sized {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> Result<usize, ()>;
}

impl<T: EncodeBE> TTFEncode for T
{
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> Result<usize, ()> {
        let start = buf.bytes.len();
        let end = start + T::PACKED_LEN;

        buf.bytes.resize(end, 0u8);
        self.encode_as_be_bytes(&mut buf.bytes[start..end]);

        Ok(start)
    }
}
