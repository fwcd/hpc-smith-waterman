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
        // helper matrix p that tracks the previous index.

        let mut h = vec![0; size];
        let mut e = vec![0; size];
        let mut f = vec![0; size];
        let mut p = vec![0; size];

        // Additionally, create helper matrices is and js that
        // map each index in our new diagonal-major scheme to
        // the original index.

        let mut is = vec![0; size];
        let mut js = vec![0; size];

        let ph = UnsafeSlice::new(&mut h);
        let pe = UnsafeSlice::new(&mut e);
        let pf = UnsafeSlice::new(&mut f);
        let pp = UnsafeSlice::new(&mut p);
        let pis = UnsafeSlice::new(&mut is);
        let pjs = UnsafeSlice::new(&mut js);

        // Perform scoring stage (dynamic programming-style)
        // We iterate over the diagonals and parallelize over
        // each element in the diagonal.
        //
        // Our matrices are now layed out as follows:
        // 
        //     | (0, 0) | (0, 1) | (1, 0) | (2, 0) | (1, 1) | ...
        //     \ k = 0 / \     k = 1     / \         k = 2
        //

        let mut previous_previous_size = 1;
        let mut previous_size = 2;
        let mut steps_since_in_bottom_part = 0;
        let mut offset = 3; // We skip the first two diagonals, see below

        // We start at 2 since the first interesting (non-border)
        // diagonal starts at i = 2 (going rightwards upwards).
        for k in 2..=(n + m) {
            // The lower and upper bounds for the diagonal('s j index)
            // Derived from rearranging the equations
            // `k - j = i < height` and `j < width` (our base range is `1..k`).
            // The outer range represent the entire j-range of the diagonal
            // whereas the inner range excludes the topmost row and the leftmost
            // column.
            let outer_lower = (k as isize - height as isize + 1).max(0) as usize;
            let outer_upper = (k + 1).min(width);
            let lower = (k as isize - height as isize + 1).max(1) as usize;
            let upper = k.min(width);
            let inner_size = upper - lower;
            let outer_size = outer_upper - outer_lower;
            // DEBUG
            // println!("k = {} ({}..{} vs {}..{}, outer: {}), stage: {}", k, lower, upper, outer_lower, outer_upper, outer_size, steps_since_in_bottom_part);
            let padding = lower - outer_lower;

            if outer_lower > 0 {
                steps_since_in_bottom_part += 1;
            }

            // Iterate the diagonal in parallel
            // TODO: Use par_iter
            (0..inner_size).into_iter().for_each(|l| {
                // Compute the 'actual'/'logical' position in the matrix.
                // We need this to index into the query/database sequence,
                // although we use our diagonal-major/cache-optimized
                // indexing scheme for the matrices instead.
                let j = lower + l;
                let i = k - j;

                // Compute indices of the neighboring cells.
                let here = offset + l + padding;
                let above = here - previous_size + if steps_since_in_bottom_part > 0 { 1 } else { 0 };
                let left = above - 1;
                let above_left = left - previous_previous_size + if steps_since_in_bottom_part > 1 { 1 } else { 0 };
                
                unsafe {
                    // Write index mappings.
                    pis.write(here, i);
                    pjs.write(here, j);

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
                }
            });

            unsafe {
                println!("diag: {:?}", &ph.slice()[(offset + padding)..(offset + outer_size - (outer_upper - upper))]);
            }

            // Store current values as previous
            previous_previous_size = previous_size;
            previous_size = outer_size;

            // Move offset
            offset += outer_size;
        }

        // DEBUG
        println!("{}", crate::utils::pretty_matrix(&h, width));

        metrics.lock().unwrap().record_cell_updates(4 * size);

        // Perform traceback stage (using the previously computed scoring matrix h)

        let mut i = (0..size).max_by_key(|&i| h[i]).unwrap();
        let mut database_indices = Vec::new();
        let mut query_indices = Vec::new();

        while i > 0 && h[i] > 0 {
            database_indices.push(is[i] - 1);
            query_indices.push(js[i] - 1);
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
