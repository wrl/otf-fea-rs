use crate::compile_model::lookup::*;

pub mod lookup;
pub use lookup::*;

pub type GPOS = LookupTable<GPOSLookup>;
