use crate::model::{Sequence, AlignedPair};

use super::Engine;

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm (naively) on the CPU.
pub struct NaiveEngine {
    g_init: i16,
    g_ext: i16,
}

impl NaiveEngine {
    /// Computes the weight between two sequence elements.
    fn weight(q: u8, d: u8) -> i16 {
        if q == d {
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
    fn align(&self, database: Sequence, query: Sequence) -> AlignedPair {
        let m = query.len();
        let n = database.len();
        let width = m + 1;
        let height = n + 1;
        let size = width * height;

        // Create scoring matrix h and helper matrices e and f

        let mut h = vec![0; size];
        let mut e = vec![0; size];
        let mut f = vec![0; size];

        // Perform scoring stage (dynamic programming-style)

        for i in 1..=m {
            for j in 1..=n {
                e[i * width + j] = (e[i * width + j - 1] - self.g_ext)
                               .max(h[i * width + j - 1] - self.g_init);
                f[i * width + j] = (f[(i - 1) * width + j] - self.g_ext)
                               .max(h[(i - 1) * width + j] - self.g_init);
                h[i * width + j] = (h[(i - 1) * width + j - 1] + Self::weight(query[i - 1], database[j - 1]))
                               .max(e[i * width + j])
                               .max(f[i * width + j])
                               .max(0);
            }
        }

        // Perform traceback stage (using the previously computed scoring matrix h)

        todo!()
    }
}
