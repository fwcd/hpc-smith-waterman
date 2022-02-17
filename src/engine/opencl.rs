use std::sync::{Arc, Mutex};
use ocl::{ProQue, Buffer, SpatialDims, OclPrm, builders::BufferBuilder, core::{MEM_WRITE_ONLY, MEM_READ_ONLY}};

use crate::{model::{Sequence, AlignedPair}, metrics::Metrics};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm with OpenCL on the
/// GPU.
pub struct OpenCLEngine {
    pro_que: ProQue,
}

impl Default for OpenCLEngine {
    fn default() -> Self {
        // The OpenCL program source code.
        let program_src: String = include_str!("program.cl")
            .replace("G_EXT", &G_EXT.to_string())
            .replace("G_INIT", &G_INIT.to_string())
            .replace("WEIGHT_IF_EQ", &WEIGHT_IF_EQ.to_string());

        // Create the program, queue and context.
        let pro_que = ProQue::builder()
            .src(program_src)
            .build()
            .unwrap();

        Self { pro_que }
    }
}

impl OpenCLEngine {
    /// Allocates a buffer of the specified length on the GPU.
    fn gpu_buffer_builder<T>(&self, len: impl Into<SpatialDims>) -> BufferBuilder<T> where T: OclPrm {
        Buffer::builder()
            .queue(self.pro_que.queue().clone())
            .len(len)
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
        let gpu_database: Buffer<u8> = self.gpu_buffer_builder(n).flags(MEM_READ_ONLY).build().unwrap();
        let gpu_query: Buffer<u8> = self.gpu_buffer_builder(m).flags(MEM_READ_ONLY).build().unwrap();
        let gpu_h: Buffer<i16> = self.gpu_buffer_builder(size).build().unwrap();
        let gpu_e: Buffer<i16> = self.gpu_buffer_builder(size).build().unwrap();
        let gpu_f: Buffer<i16> = self.gpu_buffer_builder(size).build().unwrap();
        let gpu_p: Buffer<usize> = self.gpu_buffer_builder(size).flags(MEM_WRITE_ONLY).build().unwrap();

        // Copy database and query to GPU.
        gpu_database.write(&database.raw).enq().unwrap();
        gpu_query.write(&query.raw).enq().unwrap();

        for k in 2..=(n + m) {
            // The lower and upper bounds for the diagonal
            // Derived from rearranging the equations
            // `k - j < height` and `j < width` (our base range is `1..k`).
            let lower = (k as isize - height as isize + 1).max(1) as usize;
            let upper = k.min(width);

            // Create the kernel.
            let kernel = self.pro_que.kernel_builder("smith_waterman_diagonal")
                .arg(k)
                .arg(width)
                .arg(lower)
                .arg(&gpu_database)
                .arg(&gpu_query)
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
