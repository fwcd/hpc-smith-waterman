use std::sync::{Arc, Mutex};
use ocl::{ProQue, Buffer, Kernel};
use rayon::prelude::*;

use crate::{model::{Sequence, AlignedPair, AlignedSequence}, metrics::Metrics, utils::UnsafeSlice};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// The OpenCL program source code.
const PROGRAM_SRC: &str = r#"
    __kernel void add(__global float *buffer, float scalar) {
        buffer[get_global_id(0)] += scalar;
    }
"#;

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

impl Engine for OpenCLEngine {
    fn name() -> &'static str {
        "OpenCL (GPU)"
    }

    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence, metrics: &Arc<Mutex<Metrics>>) -> AlignedPair<'a> {
        // Allocate a buffer on the GPU
        let gpu_buffer = Buffer::builder()
            .queue(self.pro_que.queue().clone())
            .len(1 << 10)
            .build()
            .unwrap();

        // Create the kernel.
        let kernel = self.pro_que.kernel_builder("add")
            .arg(&gpu_buffer)
            .arg(10f32)
            .global_work_size(1 << 5)
            .build()
            .unwrap();

        unsafe { kernel.enq().unwrap(); }

        let mut vec = vec![0f32; gpu_buffer.len()];
        gpu_buffer.read(&mut vec).enq().unwrap();
        println!("Got {:?}", vec);

        // TODO
        todo!()
    }
}
