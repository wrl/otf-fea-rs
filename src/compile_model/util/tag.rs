use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::parse_model::Tag;

impl PackedSize for Tag {
    const PACKED_LEN: usize = 4;
}

impl EncodeBE for Tag {
    #[inline]
    fn encode_as_be_bytes(&self, bytes: &mut [u8]) {
        let repr:u32 =
              (self.0[0].as_byte() as u32) << 24
            | (self.0[1].as_byte() as u32) << 16
            | (self.0[2].as_byte() as u32) << 8
            | (self.0[3].as_byte() as u32);

        bytes.copy_from_slice(&(repr.to_be_bytes()));
    }
}

impl DecodeBE for Tag {
    #[inline]
    fn decode_from_be_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);

        // FIXME: can't be unwrap()ing
        Tag::from_bytes(&arr).unwrap()
    }
}

#[macro_export]
macro_rules! tag {
    ($a:ident, $b:ident, $c:ident, $d:ident) => {
        $crate::parse_model::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::$c,
            ascii::AsciiChar::$d
        ])
    };

    ($a:ident, $b:ident, $c:ident) => {
        $crate::parse_model::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::$c,
            ascii::AsciiChar::Space
        ])
    };

    ($a:ident, $b:ident) => {
        $crate::parse_model::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space
        ])
    };

    ($a:ident) => {
        $crate::parse_model::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space
        ])
    };
}
