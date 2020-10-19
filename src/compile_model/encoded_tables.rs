use std::collections::BTreeMap;
use std::cmp::Ordering;

use endian_codec::{PackedSize, EncodeBE};

use crate::*;
use crate::Tag;
use crate::compile_model::*;
use crate::compile_model::util;

use super::tables;


#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct EncodedTableTag(Tag);

impl PartialOrd for EncodedTableTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for EncodedTableTag {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == tag!(h,e,a,d) {
            return Ordering::Less;
        }

        self.0.cmp(&other.0)
    }
}

pub struct EncodedTables {
    tables: BTreeMap<EncodedTableTag, Vec<u8>>,
    pub(crate) head: Option<tables::Head>
}

fn table_len<T: PackedSize>(_: &T) -> usize {
    return util::align_len(T::PACKED_LEN);
}

fn write_into<T: PackedSize + EncodeBE>(v: &mut Vec<u8>, p: &T) {
    let start = v.len();
    v.resize(util::align_len(start + table_len(p)), 0u8);
    p.encode_as_be_bytes(&mut v[start..]);
}

impl EncodedTables {
    pub fn new(head: Option<tables::Head>) -> Self {
        Self {
            tables: BTreeMap::new(),
            head
        }
    }

    pub fn add_table(&mut self, tag: Tag, encoded: Vec<u8>) {
        self.tables.insert(EncodedTableTag(tag), encoded);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&Tag, &Vec<u8>)> + 'a {
        self.tables.iter()
            .map(|(tag, bytes)| (&tag.0, bytes))
    }

    pub fn len(&self) -> usize {
        self.tables.len()
    }

    pub fn encode_ttf_file(&mut self, buf: &mut Vec<u8>) {
        let offset_table = TTFOffsetTable::new(TTFVersion::TTF, self.tables.len() as u16);
        write_into(buf, &offset_table);

        let mut offset = util::align_len(buf.len() +
            (self.tables.len() * TTFTableRecord::PACKED_LEN));
        let mut running_checksum = 0u32;

        for (tag, encoded) in self.tables.iter() {
            let checksum = util::checksum(encoded);

            let record = TTFTableRecord {
                tag: tag.0,
                checksum,
                offset_from_start_of_file: offset as u32,
                length: encoded.len() as u32
            };

            write_into(buf, &record);

            offset += util::align_len(encoded.len());
            running_checksum = running_checksum.overflowing_add(checksum).0;
        }

        buf.resize(util::align_len(buf.len()), 0u8);

        if let Some(ref mut head) = self.head {
            head.checksum_adjustment = {
                let whole_file_checksum = util::checksum(&buf);

                0xB1B0AFBAu32
                    .overflowing_sub(
                        whole_file_checksum
                        .overflowing_add(running_checksum).0)
                    .0
            };
        }

        for (_, encoded) in self.tables.iter() {
            buf.extend(encoded.iter());
            buf.resize(util::align_len(buf.len()), 0u8);
        }
    }
}

