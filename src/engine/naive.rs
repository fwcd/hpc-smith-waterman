use std::sync::{Arc, Mutex};

use crate::{model::{Sequence, AlignedPair, AlignedSequence}, metrics::Metrics};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm (naively) on the CPU.
pub struct NaiveEngine;

impl Default for NaiveEngine {
    fn default() -> Self {
        Self
    }
}

impl NaiveEngine {
    fn weight(d: u8, q: u8) -> i16 {
        if d == q { WEIGHT_IF_EQ } else { -WEIGHT_IF_EQ }
    }
}

impl Engine for NaiveEngine {
    fn name() -> &'static str {
        "Naive (CPU)"
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

        // Perform scoring stage (dynamic programming-style)

        for i in 1..=n {
            for j in 1..=m {
                // Compute indices for the neighboring cells
                let here = i * width + j;
                let above = (i - 1) * width + j;
                let left = i * width + j - 1;
                let above_left = (i - 1) * width + j - 1;

                // Compute helper values
                e[here] = (e[left] - G_EXT)
                      .max(h[left] - G_INIT);
                f[here] = (f[above] - G_EXT)
                      .max(h[above] - G_INIT);

                // Compute value and the remember the index the maximum came from
                // (we need this later for the traceback phase)
                let (previous, value) = [
                    (0,          0),
                    (above_left, h[above_left] + Self::weight(database[i - 1], query[j - 1])),
                    (left,       e[here]),
                    (above,      f[here]),
                ].into_iter().max_by_key(|&(_, x)| x).unwrap();
                
                h[here] = value;
                p[here] = previous;
            }
        }

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
