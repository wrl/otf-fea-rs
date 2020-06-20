use endian_codec::{PackedSize, EncodeBE, DecodeBE};
use encoding_rs::UTF_16BE;

use crate::compile_model::util::decode::*;
use crate::parse_model as pm;

#[derive(Debug)]
pub struct Name(pub Vec<NameRecord>);

impl Name {
    pub fn from_parsed_table(statements: &[pm::TableStatement]) -> Self {
        let records = statements.iter().filter_map(|s| {
            if let pm::TableStatement::NameId(n) = s {
                Some(NameRecord::from_parse_model(&n))
            } else {
                None
            }
        });

        Self(records.collect())
    }

    pub fn to_be(&self) -> Vec<u8> {
        let mut res = Vec::new();
        let records_size = self.0.len() * EncodedNameRecord::PACKED_LEN;
        let string_pool_offset = 6 + records_size;

        res.extend(&0u16.to_be_bytes()); // format
        res.extend(&(self.0.len() as u16).to_be_bytes()); // count
        res.extend(&(string_pool_offset as u16).to_be_bytes()); // string pool offset

        let records_start_offset = res.len();

        // we need to allocate the space for the records before we `encode_as_be_bytes` into the
        // vec directly
        res.resize(res.len() + records_size, 0u8);

        for (i, nr) in self.0.iter().enumerate() {
            let start = res.len();

            res.reserve(nr.name.as_bytes().len());
            for c in nr.name.encode_utf16() {
                res.extend(&c.to_be_bytes());
            }

            let record = EncodedNameRecord {
                platform_id: nr.platform_id,
                encoding_id: nr.encoding_id,
                language_id: nr.language_id,
                name_id: nr.name_id,
                string_length: (res.len() - start) as u16,
                offset: (start - string_pool_offset) as u16
            };

            record.encode_as_be_bytes(&mut res[
                (records_start_offset + (i * EncodedNameRecord::PACKED_LEN))..]);
        }

        res.shrink_to_fit();
        res
    }

    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let _format = decode_u16_be(bytes, 0);
        let count = decode_u16_be(bytes, 2);
        let string_offset = decode_u16_be(bytes, 4) as usize;

        let string_storage = &bytes[string_offset..];

        let records = decode_from_pool(count, &bytes[6..])
            .map(|r| NameRecord::from_encoded(r, string_storage))
            .collect();

        // FIXME: deal with format #1 with the lang tag records.
        //        i can't find any fonts that use format one, so leaving for later.

        Self(records)
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct EncodedNameRecord {
    platform_id: u16,
    encoding_id: u16,
    language_id: u16,
    name_id: u16,
    string_length: u16,
    offset: u16
}

#[derive(Debug)]
pub struct NameRecord {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub language_id: u16,
    pub name_id: u16,
    pub name: String
}

impl NameRecord {
    fn from_encoded(e: EncodedNameRecord, string_storage: &[u8]) -> Self {
        let EncodedNameRecord {
            platform_id,
            encoding_id,
            language_id,
            name_id,
            ..
        } = e;

        let start = e.offset as usize;
        let end = start + (e.string_length as usize);

        Self {
            platform_id,
            encoding_id,
            language_id,
            name_id,
            name:
                UTF_16BE.decode_without_bom_handling(&string_storage[start..end])
                    .0.into_owned()
        }
    }

    fn from_parse_model(n: &pm::tables::name::NameId) -> Self {
        Self {
            name_id: n.name_id as u16,
            platform_id: n.platform_id as u16,
            encoding_id: n.platform_enc_id as u16,
            language_id: n.language_id as u16,
            name: n.name.clone()
        }
    }
}
