use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use crate::{model::{Sequence, AlignedPair, AlignedSequence}, metrics::Metrics, utils::UnsafeSlice};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm with parallelized
/// diagonals on the CPU.
pub struct DiagonalEngine;

impl DiagonalEngine {
    fn weight(d: u8, q: u8) -> i16 {
        if d == q { WEIGHT_IF_EQ } else { -WEIGHT_IF_EQ }
    }
}

impl Engine for DiagonalEngine {
    fn name(&self) -> String {
        "Diagonal (CPU)".to_owned()
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

        // We start at 2 since the first interesting (non-border)
        // diagonal starts at i = 2 (going rightwards upwards).
        for k in 2..=(n + m) {
            // The lower and upper bounds for the diagonal('s j index)
            // Derived from rearranging the equations
            // `k - j = i < height` and `j < width` (our base range is `1..k`).
            let lower = (k as isize - height as isize + 1).max(1) as usize;
            let upper = k.min(width);

            // DEBUG
            let mut debug_diag = Vec::new();

            // Iterate the diagonal in parallel
            // TODO: par_iter
            (lower..upper).into_iter().for_each(|j| {
                let i = k - j;

                // Compute indices of the neighboring cells
                let here = i * width + j;
                let above = (i - 1) * width + j;
                let left = i * width + j - 1;
                let above_left = (i - 1) * width + j - 1;

                unsafe {
                    // Compute helper values
                    pe.write(here, (pe.read(left) - G_EXT).max(ph.read(left) - G_INIT));
                    pf.write(here, (pf.read(above) - G_EXT).max(ph.read(above) - G_INIT));

                    // Compute value and remember the index the maximum came from
                    // (we need this later for the traceback phase)
                    let (max_origin, max_value) = [
                        (0,          0),
                        (above_left, ph.read(above_left) + Self::weight(database[i - 1], query[j - 1])),
                        (left,       pe.read(here)),
                        (above,      pf.read(here)),
                    ].into_iter().max_by_key(|&(_, x)| x).unwrap();
                    
                    ph.write(here, max_value);
                    pp.write(here, max_origin);

                    debug_diag.push(max_value);
                }
            });
            println!("diag: {:?}", debug_diag);
        }

        // DEBUG
        println!("{}", crate::utils::pretty_matrix(&h, width));

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
