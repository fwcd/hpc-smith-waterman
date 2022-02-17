use std::sync::{Arc, Mutex};
use ocl::{ProQue, Buffer, Kernel, SpatialDims, OclPrm};
use rayon::prelude::*;

use crate::{model::{Sequence, AlignedPair, AlignedSequence}, metrics::Metrics, utils::UnsafeSlice};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// The OpenCL program source code.
const PROGRAM_SRC: &str = include_str!("program.cl");

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm with OpenCL on the
/// GPU.
pub struct OpenCLEngine {
    pro_que: ProQue,
}

impl Default for OpenCLEngine {
    fn default() -> Self {
        // Create the program, queue and context.
        let pro_que = ProQue::builder()
            .src(PROGRAM_SRC)
            .build()
            .unwrap();

        Self { pro_que }
    }
}

impl OpenCLEngine {
    /// Allocates a buffer of the specified length on the GPU.
    fn make_gpu_buffer<T>(&self, len: impl Into<SpatialDims>) -> Buffer<T> where T: OclPrm {
        Buffer::builder()
            .queue(self.pro_que.queue().clone())
            .len(len)
            .build()
            .unwrap()
    }
}

impl Engine for OpenCLEngine {
    fn name() -> &'static str {
        "OpenCL (GPU)"
    }

    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence, metrics: &Arc<Mutex<Metrics>>) -> AlignedPair<'a> {
        let n = database.len();
        let m = query.len();
        let height = n + 1;
        let width = m + 1;
        let size = height * width;

        // Allocate buffers on the GPU.
        let gpu_h: Buffer<i16> = self.make_gpu_buffer(size);
        let gpu_e: Buffer<i16> = self.make_gpu_buffer(size);
        let gpu_f: Buffer<i16> = self.make_gpu_buffer(size);
        let gpu_p: Buffer<usize> = self.make_gpu_buffer(size);

        for k in 2..=(n + m) {
            // The lower and upper bounds for the diagonal
            // Derived from rearranging the equations
            // `k - j < height` and `j < width` (our base range is `1..k`).
            let lower = (k as isize - height as isize + 1).max(1) as usize;
            let upper = k.min(width);

            // Create the kernel.
            let kernel = self.pro_que.kernel_builder("smith_waterman_diagonal")
                .arg(G_EXT)
                .arg(G_INIT)
                .arg(k)
                .arg(width)
                .arg(lower)
                .arg(&gpu_h)
                .arg(&gpu_e)
                .arg(&gpu_f)
                .arg(&gpu_p)
                .global_work_size(upper - lower)
                .build()
                .unwrap();

            // Enqueue the kernel.
            unsafe { kernel.enq().unwrap(); }
        }

        metrics.lock().unwrap().record_cell_updates(4 * size);

        // Read GPU buffers to CPU memory
        let mut h = vec![0; size];
        let mut p = vec![0; size];

        gpu_h.read(&mut h).enq().unwrap();
        gpu_p.read(&mut p).enq().unwrap();

        println!("Got {:?}", h);

        // TODO
        todo!()
    }
}
