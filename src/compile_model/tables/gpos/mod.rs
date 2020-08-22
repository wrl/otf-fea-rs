use endian_codec::{PackedSize, DecodeBE};

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;

use crate::parse_model::Tag;

pub mod header;
use header::*;

pub mod lookup;
pub use lookup::*;

#[derive(Debug)]
pub struct GPOS {
    pub script_list: ScriptList,
    pub feature_list: FeatureList,
    pub lookup_list: LookupList<GPOSSubtable>,
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

    pub fn find_lookup(&self, feature_tag: &Tag, lookup_type: u16) -> Option<usize> {
        let indices = self.feature_list.indices_for_tag(feature_tag);

        for i in indices {
            let i = *i as usize;

            match self.lookup_list.0.get(i) {
                Some(Lookup { lookup_type: lt, .. })
                    if *lt == lookup_type => return Some(i),

                _ => continue
            }
        }

        None
    }

    pub fn find_or_insert_lookup<'a>(&'a mut self, feature_tag: &Tag, lookup_type: u16)
            -> &'a mut Lookup<GPOSSubtable> {
        let idx = match self.find_lookup(feature_tag, lookup_type) {
            Some(idx) => idx,
            None => {
                let indices = self.feature_list.indices_for_tag_mut(feature_tag);
                let idx = self.lookup_list.0.len();

                indices.push(idx as u16);
                self.lookup_list.0.push(Lookup::new(lookup_type));

                idx
            }
        };

        &mut self.lookup_list.0[idx]
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

        Ok(GPOS {
            script_list: ScriptList::ttf_decode(&bytes[offsets.script..])?,
            feature_list: FeatureList::ttf_decode(&bytes[offsets.feature..])?,
            lookup_list: LookupList::ttf_decode(&bytes[offsets.lookup..])?,
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
            script: buf.append(&self.script_list)?,
            feature: buf.append(&self.feature_list)?,
            lookup: buf.append(&self.lookup_list)?,
            feature_variations: None
        };

        let header: Header_1_0 = offsets.into();
        buf.encode_at(&header, start)?;

        Ok(start)
    }
}
