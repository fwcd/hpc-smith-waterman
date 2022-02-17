mod engine;
mod fasta;
mod metrics;
mod model;
mod utils;

use clap::Parser;
use std::{io::{BufReader, self, Write}, fs::File, sync::{Mutex, Arc}, env};
use rayon::prelude::*;

use engine::{NaiveEngine, Engine, DiagonalEngine};
use fasta::FastaReader;
use metrics::Metrics;
use model::{Sequence, AlignedPair};

use crate::{utils::pretty_box, engine::OpenCLEngine};

fn run<'a, E>(database: &'a Sequence, query: &'a Sequence) -> AlignedPair<'a> where E: Default + Engine {
    let engine = E::default();
    println!("{}", pretty_box(E::name()));

    let aligned = engine.align(database, query, &Arc::new(Mutex::new(Metrics::new())));
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);

    aligned
}

fn bench_sequential<'a, E>(database: &'a Sequence, queries: &'a Vec<Sequence>) -> Vec<AlignedPair<'a>> where E: Default + Engine {
    let engine = E::default();
    println!("{}", pretty_box(format!("{} (sequential)", E::name())));

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

fn bench_parallel<'a, E>(database: &'a Sequence, queries: &'a Vec<Sequence>) -> Vec<AlignedPair<'a>> where E: Default + Engine + Sync {
    let engine = E::default();
    println!("{}", pretty_box(format!("{} (parallel)", E::name())));

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

    /// Whether to benchmark the naive (CPU) engine.
    #[clap(long)]
    naive: bool,

    /// Whether to benchmark the diagonal (CPU) engine.
    #[clap(long)]
    diagonal: bool,

    /// Whether to benchmark the OpenCL (GPU) engine.
    #[clap(long)]
    opencl: bool,
}

fn main() {
    let args = Args::parse();
    let default = env::args().len() == 1;

    // Run short demo if --demo is set
    if args.demo || default {
        let demo_database = "TGTTACGG".parse().unwrap();
        let demo_query = "GGTTGACTA".parse().unwrap();
        run::<NaiveEngine>(&demo_database, &demo_query);
    }

    // Read a subset of the sequences from the downloaded dataset
    let file = File::open(args.path).expect("Could not open dataset (did you specify --path?)");
    let mut reader = FastaReader::new(BufReader::new(file));
    let database = reader.next().unwrap();
    let queries = reader.take(10_000).collect();
    let mut all_aligns = Vec::new();

    // Benchmark the naive (CPU) engine
    if args.naive || default {
        all_aligns.push(bench_sequential::<NaiveEngine>(&database, &queries));
        all_aligns.push(bench_parallel::<NaiveEngine>(&database, &queries));
    }

    // Benchmark the diagonal (CPU) engine
    if args.diagonal || default {
        all_aligns.push(bench_parallel::<DiagonalEngine>(&database, &queries));
    }

    // Benchmark the OpenCL (GPU) engine
    if args.opencl || default {
        all_aligns.push(bench_parallel::<OpenCLEngine>(&database, &queries));
    }

    // Assert that all benchmarks yielded the same result
    assert!(all_aligns.windows(2).all(|w| w[0] == w[1]));
}
