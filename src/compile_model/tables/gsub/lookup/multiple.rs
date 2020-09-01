use std::ops;

use crate::compile_model::coverage::*;


type inner = CoverageLookup<Vec<u16>>;

#[derive(Debug, Default)]
pub struct Multiple(pub inner);

impl ops::Deref for Multiple {
    type Target = inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Multiple {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
