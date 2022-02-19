mod constants;
mod naive;
mod diagonal;
mod opencl;

pub use constants::*;
pub use naive::*;
pub use diagonal::*;
pub use opencl::*;

use std::sync::{Arc, Mutex};

use crate::{model::{Sequence, AlignedPair}, metrics::Metrics};

/// A facility that computes the alignment of two sequences.
pub trait Engine {
    /// The engine's name.
    fn name(&self) -> String;

    /// Aligns the given two sequences.
    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence, metrics: &Arc<Mutex<Metrics>>) -> AlignedPair<'a>;
}
