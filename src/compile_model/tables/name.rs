use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use encoding_rs::UTF_16BE;

#[derive(Debug)]
pub struct Name(pub Vec<NameRecord>);

fn decode_be(bytes: &[u8], offset: usize) -> u16 {
    let mut a = [0u8; 2];
    a.copy_from_slice(&bytes[offset..offset+2]);
    u16::from_be_bytes(a)
}

impl Name {
    #[inline]
    pub fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let mut records = Vec::new();

        let format = decode_be(bytes, 0);
        let count = decode_be(bytes, 2) as usize;
        let string_offset = decode_be(bytes, 4) as usize;

        println!(" >> format {}", format);

        let string_storage = &bytes[string_offset..];

        for i in 0..count {
            let start = 6 + (i as usize * EncodedNameRecord::PACKED_LEN);
            let end = start + EncodedNameRecord::PACKED_LEN;

            let decoded = EncodedNameRecord::decode_from_be_bytes(
                &bytes[start..end]);

            records.push(
                NameRecord::from_encoded(decoded, string_storage));
        }

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
}
