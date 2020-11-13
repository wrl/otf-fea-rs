use std::collections::{
    HashMap,
    HashSet
};

use crate::SourcePosition;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompiledEntry {
    I16(usize)
}

pub type SourceMap = HashMap<SourcePosition, HashSet<CompiledEntry>>;
