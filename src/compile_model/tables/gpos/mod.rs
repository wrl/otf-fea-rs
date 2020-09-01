use std::collections::HashMap;

use endian_codec::{PackedSize, DecodeBE};

use crate::*;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::script_list::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::lookup_list::*;

use crate::parse_model::LookupName;


pub mod header;
use header::*;

pub mod lookup;
pub use lookup::*;


#[derive(Debug)]
pub struct LookupTable<L: Sized> {
    pub script_list: ScriptList,
    pub feature_list: FeatureList,
    pub lookup_list: LookupList<L>,
    pub feature_variations: Option<usize>,

    pub named_lookups: HashMap<LookupName, Vec<u16>>
}

impl<L> LookupTable<L> {
    pub fn lookup_index_for_type<T, I>(&self, indices: I) -> Option<usize>
        where T: LookupSubtable<L>,
              I: IntoIterator<Item = usize>
    {
        for i in indices {
            if let Some(_) = self.lookup_list.0.get(i).map(T::get_lookup_variant) {
                return Some(i);
            }
        }

        None
    }
}

pub trait KeyedLookups<K, L> {
    fn find_lookup<T>(&mut self, lookup_key: &K) -> Option<usize>
        where T: LookupSubtable<L>;

    fn find_or_insert_lookup<'a, T>(&'a mut self, lookup_key: &K) -> &'a mut Lookup<T>
        where T: LookupSubtable<L>;
}

impl<L> KeyedLookups<LookupName, L> for LookupTable<L> {
    fn find_lookup<T>(&mut self, lookup_name: &LookupName) -> Option<usize>
        where T: LookupSubtable<L>
    {
        self.named_lookups.get(lookup_name)
            .and_then(|indices| {
                self.lookup_index_for_type::<T, _> (indices.iter().map(|x| *x as usize))
            })
    }

    fn find_or_insert_lookup<'a, T>(&'a mut self, lookup_name: &LookupName) -> &'a mut Lookup<T>
        where T: LookupSubtable<L>
    {
        let idx = match self.find_lookup::<T>(lookup_name) {
            Some(idx) => idx,
            None => {
                let idx = self.lookup_list.0.len();
                self.lookup_list.0.push(T::new_lookup());

                self.named_lookups.entry(lookup_name.clone())
                    .or_default()
                    .push(idx as u16);

                idx
            }
        };

        // unwrap() is fine here since we've either already succeeded with T::get_lookup_variant()
        // in find_lookup() or T::new_lookup() has inserted a valid lookup.
        //
        // it's possible for T::new_lookup() to create a lookup which is not then matched by
        // T::get_lookup_variant_mut(), but that's a programmer error that the panic from unwrap
        // will direct the programmer to fix the issue.
        T::get_lookup_variant_mut(&mut self.lookup_list.0[idx]).unwrap()
    }
}

impl<L> KeyedLookups<FeatureTag, L> for LookupTable<L> {
    fn find_lookup<T>(&mut self, feature_tag: &FeatureTag) -> Option<usize>
        where T: LookupSubtable<L>
    {
        self.lookup_index_for_type::<T, _>(
            self.feature_list.indices_for_tag(feature_tag).iter()
                .map(|x| *x as usize))
    }

    fn find_or_insert_lookup<'a, T>(&'a mut self, feature_tag: &FeatureTag) -> &'a mut Lookup<T>
        where T: LookupSubtable<L>
    {
        let idx = match self.find_lookup::<T>(feature_tag) {
            Some(idx) => idx,
            None => {
                let idx = self.lookup_list.0.len();

                self.feature_list.add_lookup_index(feature_tag, idx as u16);
                self.lookup_list.0.push(T::new_lookup());

                idx
            }
        };

        // unwrap() is fine here since we've either already succeeded with T::get_lookup_variant()
        // in find_lookup() or T::new_lookup() has inserted a valid lookup.
        //
        // it's possible for T::new_lookup() to create a lookup which is not then matched by
        // T::get_lookup_variant_mut(), but that's a programmer error that the panic from unwrap
        // will direct the programmer to fix the issue.
        T::get_lookup_variant_mut(&mut self.lookup_list.0[idx]).unwrap()
    }
}

pub type GPOS = LookupTable<GPOSLookup>;

impl GPOS {
    pub fn new() -> Self {
        LookupTable {
            script_list: ScriptList::new(),
            feature_list: FeatureList::new(),
            lookup_list: LookupList::new(),
            feature_variations: None,

            named_lookups: HashMap::new()
        }
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

        Ok(LookupTable {
            script_list: ScriptList::ttf_decode(script_bytes, feature_bytes)?,
            feature_list: FeatureList::ttf_decode(feature_bytes)?,
            lookup_list: LookupList::ttf_decode(lookup_bytes)?,
            feature_variations: offsets.feature_variations,

            named_lookups: HashMap::new()
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
