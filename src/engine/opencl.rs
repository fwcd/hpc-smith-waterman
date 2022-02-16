use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use crate::{model::{Sequence, AlignedPair, AlignedSequence}, metrics::Metrics, utils::UnsafeSlice};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm with OpenCL on the
/// GPU.
pub struct OpenCLEngine;

impl Default for OpenCLEngine {
    fn default() -> Self {
        Self
    }
}

impl Engine for OpenCLEngine {
    fn name() -> &'static str {
        "OpenCL (GPU)"
    }

    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence, metrics: &Arc<Mutex<Metrics>>) -> AlignedPair<'a> {
        // TODO
        todo!()
    }
}
