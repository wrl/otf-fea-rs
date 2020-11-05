use std::collections::{
    HashMap,
    HashSet
};

use crate::SourceSpan;


#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CompiledEntry {
    I16(usize)
}

pub type SourceMap = HashMap<SourceSpan, HashSet<CompiledEntry>>;
