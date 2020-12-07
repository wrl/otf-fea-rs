use std::collections::{
    HashMap,
    BTreeMap,
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompiledEntry {
    I16(usize)
}

pub type SourceMap = HashMap<usize, BTreeMap<usize, CompiledEntry>>;
