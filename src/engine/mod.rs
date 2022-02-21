mod constants;
mod naive;
mod diagonal;
mod optimized_diagonal;
mod opencl_diagonal;

pub use constants::*;
pub use naive::*;
pub use diagonal::*;
pub use optimized_diagonal::*;
pub use opencl_diagonal::*;

use std::sync::{Arc, Mutex};

use crate::{model::{Sequence, AlignedPair}, metrics::Metrics};

/// A facility that computes the alignment of two sequences.
pub trait Engine {
    /// The engine's name.
    fn name(&self) -> String;

    /// Aligns the given two sequences.
    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence, metrics: &Arc<Mutex<Metrics>>) -> AlignedPair<'a>;
}
