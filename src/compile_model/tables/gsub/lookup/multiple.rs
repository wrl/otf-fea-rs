use crate::compile_model::coverage::*;


#[derive(Debug, Default)]
pub struct Multiple(pub CoverageLookup<Vec<u16>>);
