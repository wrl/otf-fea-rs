#![allow(non_camel_case_types)]

pub mod tables;

mod class_def;
pub use class_def::ClassDef;

mod coverage;
pub use coverage::CoverageLookup;

mod device;
pub use device::*;

mod error;
pub use error::*;

mod feature_list;
pub use feature_list::{
    FeatureList,
    FeatureRecord,
};

pub mod lookup;
pub use lookup::{
    LookupList,
    LookupFlags,
    Lookup,

    LookupSubtable
};

mod script_list;
pub use script_list::{
    ScriptList,
    Script,
};

#[macro_use]
pub mod util;
pub use util::TTFVersion;

mod value_record;
pub use value_record::{
    ValueRecord,
    ValueRecordFromParsed
};

mod ttf_table;
pub use ttf_table::*;

pub(crate) mod compiler_state;
pub(crate) use compiler_state::CompilerState;
pub use compiler_state::CompilerOutput;
