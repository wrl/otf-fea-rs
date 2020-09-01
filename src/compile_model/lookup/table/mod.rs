use std::collections::HashMap;

use crate::*;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::feature_list::*;
use crate::compile_model::script_list::*;
use crate::compile_model::lookup::*;

use crate::parse_model::LookupName;


mod header;
mod codec;


#[derive(Debug)]
pub struct LookupTable<L: Sized>
{
    pub script_list: ScriptList,
    pub feature_list: FeatureList,
    pub lookup_list: LookupList<L>,
    pub feature_variations: Option<usize>,

    pub named_lookups: HashMap<LookupName, Vec<u16>>
}

impl<L> LookupTable<L> {
    pub fn new() -> Self {
        Self {
            script_list: ScriptList::new(),
            feature_list: FeatureList::new(),
            lookup_list: LookupList::new(),
            feature_variations: None,

            named_lookups: HashMap::new()
        }
    }

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
