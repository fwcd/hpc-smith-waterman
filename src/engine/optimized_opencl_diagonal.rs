use std::sync::{Arc, Mutex};
use ocl::{Buffer, core::{MEM_WRITE_ONLY, MEM_READ_ONLY}, Queue, Program, Context, Platform, Device, DeviceType, Kernel};

use crate::{model::{Sequence, AlignedPair, AlignedSequence}, metrics::Metrics};

use super::{Engine, G_INIT, G_EXT, WEIGHT_IF_EQ};

/// An engine that computes alignments using the
/// Smith-Waterman-Algorithm with OpenCL on the
/// GPU by parallelizing over the diagonals.
/// This variant additionally uses a diagonal-major
/// layout of the matrix for better cache performance.
pub struct OptimizedOpenCLDiagonalEngine {
    program: Program,
    device: Device,
    context: Context,
}

impl OptimizedOpenCLDiagonalEngine {
    pub fn new(gpu_index: usize) -> Self {
        // The OpenCL program source code.
        let program_src: String = include_str!("optimized_opencl_diagonal.cl")
            .replace("G_EXT", &G_EXT.to_string())
            .replace("G_INIT", &G_INIT.to_string())
            .replace("WEIGHT_IF_EQ", &WEIGHT_IF_EQ.to_string());

        // Fetch platform and device
        let platform = Platform::default();
        let device = Device::list(platform, Some(DeviceType::GPU))
            .unwrap()
            .into_iter()
            .nth(gpu_index)
            .expect("GPU not found for OpenCL");

        // Create the context
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .unwrap();

        // Create the program
        let program = Program::builder()
            .src(program_src)
            .build(&context)
            .unwrap();

        Self { program, device, context }
    }
}

impl Engine for OptimizedOpenCLDiagonalEngine {
    fn name(&self) -> String {
        format!("Optimized OpenCL Diagonal (GPU: {})", self.device.name().unwrap())
    }

    fn align<'a>(&self, database: &'a Sequence, query: &'a Sequence, metrics: &Arc<Mutex<Metrics>>) -> AlignedPair<'a> {
        let n = database.len();
        let m = query.len();
        let height = n + 1;
        let width = m + 1;
        let size = height * width;

        // Create a queue
        let queue = Queue::new(&self.context, self.device, None).unwrap();

        // Allocate buffers on the GPU.
        let gpu_database: Buffer<u8> = Buffer::builder().queue(queue.clone()).len(n).flags(MEM_READ_ONLY).build().unwrap();
        let gpu_query: Buffer<u8> = Buffer::builder().queue(queue.clone()).len(m).flags(MEM_READ_ONLY).build().unwrap();
        let gpu_h: Buffer<i16> = Buffer::builder().queue(queue.clone()).len(size).build().unwrap();
        let gpu_e: Buffer<i16> = Buffer::builder().queue(queue.clone()).len(size).build().unwrap();
        let gpu_f: Buffer<i16> = Buffer::builder().queue(queue.clone()).len(size).build().unwrap();
        let gpu_p: Buffer<u32> = Buffer::builder().queue(queue.clone()).len(size).flags(MEM_WRITE_ONLY).build().unwrap();

        // Copy database and query to GPU.
        gpu_database.write(&database.raw).enq().unwrap();
        gpu_query.write(&query.raw).enq().unwrap();

        // Create the kernel.
        let mut kernel = Kernel::builder()
            .name("smith_waterman_diagonal")
            .program(&self.program)
            .queue(queue)
            .arg(width as u32)
            .arg(&gpu_database)
            .arg(&gpu_query)
            .arg(&gpu_h)
            .arg(&gpu_e)
            .arg(&gpu_f)
            .arg(&gpu_p)
            .build()
            .unwrap();

        // TODO

        metrics.lock().unwrap().record_cell_updates(4 * size);

        // Read GPU buffers to CPU memory
        let mut h = vec![0; size];
        let mut p = vec![0; size];

        gpu_h.read(&mut h).enq().unwrap();
        gpu_p.read(&mut p).enq().unwrap();

        // Perform traceback stage (using the previously computed scoring matrix h)

        let mut i = (0..size).max_by_key(|&i| h[i]).unwrap();
        let mut database_indices = Vec::new();
        let mut query_indices = Vec::new();

        while i > 0 && h[i] > 0 {
            database_indices.push((i / width) - 1);
            query_indices.push((i % width) - 1);
            i = p[i] as usize;
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
