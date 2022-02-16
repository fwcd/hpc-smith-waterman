use crate::model::{Sequence, AlignedPair, AlignedSequence};

use super::Engine;

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm (naively) on the CPU.
pub struct NaiveEngine {
    g_init: i16,
    g_ext: i16,
}

impl NaiveEngine {
    /// Computes the weight between two sequence elements.
    fn weight(d: u8, q: u8) -> i16 {
        if d == q {
            3
        } else {
            -3
        }
    }
}

impl Default for NaiveEngine {
    /// A naive engine with the default parameters from the exercise sheet.
    fn default() -> Self {
        Self { g_init: 2, g_ext: 2 }
    }
}

impl Engine for NaiveEngine {
    fn name() -> &'static str {
        "Naive (CPU, single-threaded)"
    }

    fn align(&self, database: Sequence, query: Sequence) -> AlignedPair {
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
                e[here] = (e[left] - self.g_ext)
                      .max(h[left] - self.g_init);
                f[here] = (f[above] - self.g_ext)
                      .max(h[above] - self.g_init);

                // Compute value and the direction we came from
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

        AlignedPair::new(
            AlignedSequence::new(database, database_indices),
            AlignedSequence::new(query, query_indices),
        )
    }
}
