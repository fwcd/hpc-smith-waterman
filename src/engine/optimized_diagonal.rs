use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use crate::{model::{Sequence, AlignedPair, AlignedSequence}, metrics::Metrics, utils::UnsafeSlice};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm with parallelized
/// diagonals on the CPU. This variant additionally
/// uses a diagonal-major layout of the matrix for
/// better cache performance.
pub struct OptimizedDiagonalEngine;

impl OptimizedDiagonalEngine {
    fn weight(d: u8, q: u8) -> i16 {
        if d == q { WEIGHT_IF_EQ } else { -WEIGHT_IF_EQ }
    }
}

impl Engine for OptimizedDiagonalEngine {
    fn name(&self) -> String {
        "Optimized Diagonal (CPU)".to_owned()
    }

    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence, metrics: &Arc<Mutex<Metrics>>) -> AlignedPair<'a> {
        let n = database.len();
        let m = query.len();
        let height = n + 1;
        let width = m + 1;
        let size = height * width;

        // Create scoring matrix h, helper matrices e and f and a
        // helper matrix p that tracks the previous index

        let mut h = vec![0; size];
        let mut e = vec![0; size];
        let mut f = vec![0; size];
        let mut p = vec![0; size];

        let ph = UnsafeSlice::new(&mut h);
        let pe = UnsafeSlice::new(&mut e);
        let pf = UnsafeSlice::new(&mut f);
        let pp = UnsafeSlice::new(&mut p);

        // Perform scoring stage (dynamic programming-style)
        // We iterate over the diagonals and parallelize over
        // each element in the diagonal.

        // TODO: Implement the new inner loop
        todo!();

        metrics.lock().unwrap().record_cell_updates(4 * size);

        // Perform traceback stage (using the previously computed scoring matrix h)

        let mut i = (0..size).max_by_key(|&i| h[i]).unwrap();
        let mut database_indices = Vec::new();
        let mut query_indices = Vec::new();

        while i > 0 && h[i] > 0 {
            database_indices.push((i / width) - 1);
            query_indices.push((i % width) - 1);
            i = p[i];
        }

        database_indices.reverse();
        query_indices.reverse();

        metrics.lock().unwrap().record_sequence_pair();

        AlignedPair::new(
            AlignedSequence::new(database, database_indices),
            AlignedSequence::new(query, query_indices),
        )
    }
}
