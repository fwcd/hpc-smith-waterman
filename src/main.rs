mod engine;
mod fasta;
mod metrics;
mod model;
mod utils;

use clap::Parser;
use std::{io::{BufReader, self, Write}, fs::File, sync::{Mutex, Arc}};
use rayon::prelude::*;

use engine::{NaiveEngine, Engine, DiagonalEngine};
use fasta::FastaReader;
use metrics::Metrics;
use model::{Sequence, AlignedPair};

use crate::{utils::{pretty_box, EqualAsserter}, engine::OpenCLEngine};

fn run<'a>(engine: &impl Engine, database: &'a Sequence, query: &'a Sequence) -> AlignedPair<'a> {
    println!("{}", pretty_box(engine.name()));

    let aligned = engine.align(database, query, &Arc::new(Mutex::new(Metrics::new())));
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);

    aligned
}

fn bench_sequential<'a>(engine: &impl Engine, database: &'a Sequence, queries: &'a Vec<Sequence>) -> Vec<AlignedPair<'a>> {
    println!("{}", pretty_box(format!("{} (sequential)", engine.name())));

    let total = queries.len();
    let metrics = Arc::new(Mutex::new(Metrics::new()));
    let aligns = queries.iter().enumerate().map(|(i, query)| {
        let aligned = engine.align(database, query, &metrics);
        if i % 100 == 0 {
            print!("\r[{} %]", (i * 100) / total);
            io::stdout().flush().unwrap();
        }
        aligned
    }).collect();

    print!("\r");
    metrics.lock().unwrap().print();
    aligns
}

fn bench_parallel<'a>(engine: &(impl Engine + Sync), database: &'a Sequence, queries: &'a Vec<Sequence>) -> Vec<AlignedPair<'a>> {
    println!("{}", pretty_box(format!("{} (parallel)", engine.name())));

    let metrics = Arc::new(Mutex::new(Metrics::new()));
    let aligns = queries.par_iter().map(|query| {
        engine.align(database, query, &metrics)
    }).collect();

    metrics.lock().unwrap().print();
    aligns
}

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    /// Whether to run a short demo before the actual benchmarks.
    #[clap(short, long)]
    demo: bool,

    /// The path to the downloaded dataset.
    #[clap(short, long, default_value = "data/uniprot_sprot.fasta")]
    path: String,

    /// The maximum number of sequences to benchmark against.
    #[clap(short, long, default_value_t = 10_000)]
    number: usize,

    /// Whether to benchmark the naive (CPU) engine.
    #[clap(long)]
    naive: bool,

    /// Whether to benchmark the diagonal (CPU) engine.
    #[clap(long)]
    diagonal: bool,

    /// Whether to benchmark the OpenCL (GPU) engine.
    #[clap(long)]
    opencl: bool,

    /// The index of the GPU to use for the OpenCL engine.
    #[clap(long, default_value_t = 0)]
    gpu_index: usize,
}

fn main() {
    let args = Args::parse();
    let default = !args.demo && !args.naive && !args.diagonal && !args.opencl;

    // Create engines
    let naive_engine = NaiveEngine;
    let diagonal_engine = DiagonalEngine;
    let opencl_engine = OpenCLEngine::new(args.gpu_index);

    // Run short demo if --demo is set
    if args.demo || default {
        let demo_database = "TGTTACGG".parse().unwrap();
        let demo_query = "GGTTGACTA".parse().unwrap();
        run(&diagonal_engine, &demo_database, &demo_query);
    }

    // Read a subset of the sequences from the downloaded dataset
    let file = File::open(args.path).expect("Could not open dataset (did you specify --path?)");
    let mut reader = FastaReader::new(BufReader::new(file));
    let database = reader.next().unwrap();
    let queries = reader.take(args.number).collect();
    let mut asserter = EqualAsserter::new();

    // Benchmark the naive (CPU) engine
    if args.naive || default {
        asserter.feed("naive sequential", bench_sequential(&naive_engine, &database, &queries));
        asserter.feed("naive parallel", bench_parallel(&naive_engine, &database, &queries));
    }

    // Benchmark the diagonal (CPU) engine
    if args.diagonal || default {
        asserter.feed("diagonal parallel", bench_parallel(&diagonal_engine, &database, &queries));
    }

    // Benchmark the OpenCL (GPU) engine
    if args.opencl || default {
        asserter.feed("opencl parallel", bench_parallel(&opencl_engine, &database, &queries));
    }
}
