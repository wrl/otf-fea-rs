use crate::compile_model::lookup::*;

pub mod lookup;
pub use lookup::*;

mod anchor;
pub use anchor::*;

pub type GPOS = LookupTable<GPOSLookup>;
