use endian_codec::{PackedSize, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;

use crate::{
    Tag,
    tag
};

pub mod header;
use header::*;

pub mod lookup;
pub use lookup::*;

#[derive(Debug)]
pub struct GPOS {
    pub script_list: ScriptList,
    pub feature_list: FeatureList,
    pub lookup_list: LookupList<GPOSLookup>,
    pub feature_variations: Option<usize>
}

impl GPOS {
    pub fn new() -> Self {
        Self {
            script_list: ScriptList::new(),
            feature_list: FeatureList::new(),
            lookup_list: LookupList::new(),
            feature_variations: None
        }
    }

    pub fn find_lookup<T, G>(&mut self, feature_tag: &Tag, get_lookup_variant: G) -> Option<usize>
        where G: Fn(&mut GPOSLookup) -> Option<&mut Lookup<T>>
    {
        let indices = self.feature_list.indices_for_tag(feature_tag);

        for i in indices {
            let i = *i as usize;

            match self.lookup_list.0.get_mut(i) {
                Some(l) =>
                    if get_lookup_variant(l).is_some() {
                        return Some(i)
                    },

                _ => continue
            }
        }

        None
    }

    pub fn find_or_insert_lookup<'a, T, G, I>(&'a mut self, feature_tag: &Tag, get_lookup_variant: G, insert: I)
        -> &'a mut Lookup<T>
        where G: Fn(&mut GPOSLookup) -> Option<&mut Lookup<T>> + Copy,
              I: Fn() -> GPOSLookup
    {
        let idx = match self.find_lookup(feature_tag, get_lookup_variant) {
            Some(idx) => idx,
            None => {
                let indices = self.feature_list.indices_for_tag_mut(feature_tag);
                let idx = self.lookup_list.0.len();

                self.script_list.script_for_tag_mut(&tag!(D,F,L,T))
                    .default_lang_sys.features.push(*feature_tag);

                indices.push(idx as u16);
                self.lookup_list.0.push(insert());

                idx
            }
        };

        // unwrap() is fine here since we've either already succeeded with get_lookup_variant() in
        // find_lookup() or insert() has inserted a valid lookup.
        //
        // it's possible for insert() to create a lookup which is not then matched by
        // get_lookup_variant(), but that's a programmer error that the panic from unwrap will
        // direct the programmer to fix the issue.
        //
        // if we ever get "enum variants as types" we'll be able to use those here instead.
        get_lookup_variant(&mut self.lookup_list.0[idx]).unwrap()
    }
}

impl TTFDecode for GPOS {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let version: Version = decode_from_slice(bytes);

        let offsets: Offsets = match (version.major, version.minor) {
            (1, 0) => Header_1_0::decode_from_be_bytes(bytes).into(),
            (1, 1) => Header_1_1::decode_from_be_bytes(bytes).into(),

            _ => return Err(DecodeError::InvalidValue("version", "GPOS".into()))
        };

        let script_bytes = &bytes[offsets.script..];
        let feature_bytes = &bytes[offsets.feature..];
        let lookup_bytes = &bytes[offsets.lookup..];

        Ok(GPOS {
            script_list: ScriptList::ttf_decode(script_bytes, feature_bytes)?,
            feature_list: FeatureList::ttf_decode(feature_bytes)?,
            lookup_list: LookupList::ttf_decode(lookup_bytes)?,
            feature_variations: offsets.feature_variations
        })
    }
}

impl TTFEncode for GPOS {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let header_size =
            self.feature_variations
                .map(|_| Header_1_1::PACKED_LEN)
                .unwrap_or(Header_1_0::PACKED_LEN);

        let start = buf.bytes.len();
        buf.bytes.resize(header_size, 0u8);

        let offsets = Offsets {
            script: self.script_list.ttf_encode(buf, &self.feature_list)?,
            feature: self.feature_list.ttf_encode(buf)?,
            lookup: self.lookup_list.ttf_encode(buf)?,
            feature_variations: None
        };

        let header: Header_1_0 = offsets.into();
        buf.encode_at(&header, start)?;

        Ok(start)
    }
}
