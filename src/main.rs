mod engine;
mod fasta;
mod metrics;
mod model;
mod utils;

use clap::{Parser, Subcommand};
use std::{io::{BufReader, self, Write}, fs::File, sync::{Mutex, Arc}};
use rayon::prelude::*;

use engine::{NaiveEngine, Engine, DiagonalEngine, OptimizedDiagonalEngine, OptimizedOpenCLDiagonalEngine};
use fasta::FastaReader;
use metrics::Metrics;
use model::{Sequence, AlignedPair};

use crate::{utils::{pretty_box, EqualAsserter}, engine::OpenCLDiagonalEngine};

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

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    /// The index of the GPU to use (for OpenCL).
    #[clap(short, long)]
    gpu_index: usize,
}

#[derive(Subcommand)]
enum Command {
    /// Runs the engines once on a pair of sequences.
    Run {
        /// The database sequence.
        #[clap(default_value = "TGTTACGG")]
        database: String,
        /// The query sequence.
        #[clap(default_value = "GGTTGACTA")]
        query: String,
    },
    /// Benchmarks the different engines.
    Bench {
        /// The path to the downloaded dataset.
        #[clap(short, long, default_value = "data/uniprot_sprot.fasta")]
        path: String,

        /// The maximum number of sequences to benchmark against.
        #[clap(short, long, default_value_t = 10_000)]
        number: usize,

        /// How many times to repeat/cycle each sequence.
        #[clap(short, long, default_value_t = 1)]
        repeats: usize,

        /// Whether to benchmark the naive (CPU) engine.
        #[clap(long)]
        naive: bool,

        /// Whether to benchmark the diagonal (CPU) engine.
        #[clap(long)]
        diagonal: bool,

        /// Whether to benchmark the cache-optimized diagonal (CPU) engine.
        #[clap(long)]
        optimized_diagonal: bool,

        /// Whether to benchmark the diagonal OpenCL (GPU) engine.
        #[clap(long)]
        opencl_diagonal: bool,

        /// Whether to benchmark the cache-optimized diagonal OpenCL (GPU) engine.
        #[clap(long)]
        optimized_opencl_diagonal: bool,
    },
}

fn main() {
    // Parse CLI args
    let cli = Cli::parse();

    // Create engines
    let naive_engine = NaiveEngine;
    let diagonal_engine = DiagonalEngine;
    let optimized_diagonal_engine = OptimizedDiagonalEngine;
    let opencl_diagonal_engine = OpenCLDiagonalEngine::new(cli.gpu_index);
    let optimized_opencl_diagonal_engine = OptimizedOpenCLDiagonalEngine::new(cli.gpu_index);

    match cli.command {
        Command::Run { database, query } => {
            let database = database.parse().unwrap();
            let query = query.parse().unwrap();

            run(&naive_engine, &database, &query);
            run(&diagonal_engine, &database, &query);
            run(&optimized_diagonal_engine, &database, &query);
            run(&opencl_diagonal_engine, &database, &query);
            run(&optimized_opencl_diagonal_engine, &database, &query);
        },
        Command::Bench { path, number, repeats, naive, diagonal, opencl_diagonal, optimized_diagonal, optimized_opencl_diagonal } => {
            let default = !naive && !diagonal && !optimized_diagonal && !opencl_diagonal && !optimized_opencl_diagonal;
            // Read a subset of the sequences from the downloaded dataset
            let file = File::open(path).expect("Could not open dataset (did you specify --path?)");
            let mut reader = FastaReader::new(BufReader::new(file)).map(|x| x.cycle(repeats));
            let database = reader.next().unwrap();
            let queries = reader.take(number).collect();

            // Use asserters to verify that engines yield the same result.
            // Note that the optimized diagonal engines use a different
            // asserter since they may yield different solutions during
            // the traceback stage if there are multiple (equivalent) maximums.
            let mut asserter = EqualAsserter::new();
            let mut optimized_asserter = EqualAsserter::new();

            // Benchmark the naive (CPU) engine
            if naive || default {
                asserter.feed(bench_sequential(&naive_engine, &database, &queries));
                asserter.feed(bench_parallel(&naive_engine, &database, &queries));
            }

            // Benchmark the diagonal (CPU) engine
            if diagonal || default {
                asserter.feed(bench_parallel(&diagonal_engine, &database, &queries));
            }

            // Benchmark the cache-optimized diagonal (CPU) engine
            if optimized_diagonal || default {
                optimized_asserter.feed(bench_parallel(&optimized_diagonal_engine, &database, &queries));
            }

            // Benchmark the OpenCL diagonal (GPU) engine
            if opencl_diagonal || default {
                asserter.feed(bench_parallel(&opencl_diagonal_engine, &database, &queries));
            }

            // Benchmark the cache-optimized OpenCL diagonal (GPU) engine
            if optimized_opencl_diagonal || default {
                optimized_asserter.feed(bench_parallel(&optimized_opencl_diagonal_engine, &database, &queries));
            }
        },
    }
}
