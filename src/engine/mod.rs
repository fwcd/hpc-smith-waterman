mod naive;

pub use naive::*;

use crate::model::{Sequence, AlignedPair};

/// A facility that computes the alignment of two sequences.
pub trait Engine {
    /// The engine's name.
    fn name() -> &'static str;

    /// Aligns the given two sequences.
    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence) -> AlignedPair<'a>;
}
