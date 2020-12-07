use std::collections::{
    HashMap,
    BTreeMap,
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompiledEntry {
    I16(usize)
}

// FIXME: should be BTreeSet<CompiledEntry> in the case of one source location mapped to multiple
// compiled representations.
pub type SourceMap = HashMap<usize, BTreeMap<usize, CompiledEntry>>;
