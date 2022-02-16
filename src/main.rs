mod engine;
mod fasta;
mod metrics;
mod model;
mod utils;

use std::{io::{BufReader, self, Write}, fs::File, sync::{Mutex, Arc}};
use rayon::prelude::*;

use engine::{NaiveEngine, Engine, DiagonalEngine};
use fasta::FastaReader;
use metrics::Metrics;
use model::{Sequence, AlignedPair};

fn run<'a, E>(database: &'a Sequence, query: &'a Sequence) -> AlignedPair<'a> where E: Default + Engine {
    let engine = E::default();
    println!("=== {} ===", E::name());

    let aligned = engine.align(database, query, &Arc::new(Mutex::new(Metrics::new())));
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);

    aligned
}

fn bench_sequential<'a, E>(database: &'a Sequence, queries: &'a Vec<Sequence>) -> Vec<AlignedPair<'a>> where E: Default + Engine {
    let engine = E::default();
    println!("=== {} (sequential) ===", E::name());

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
    println!("=== {} (parallel) ===", E::name());

    let metrics = Arc::new(Mutex::new(Metrics::new()));
    let aligns = queries.par_iter().map(|query| {
        engine.align(database, query, &metrics)
    }).collect();

    metrics.lock().unwrap().print();
    aligns
}

fn main() {
    let demo_database = "TGTTACGG".parse().unwrap();
    let demo_query = "GGTTGACTA".parse().unwrap();
    run::<NaiveEngine>(&demo_database, &demo_query);
    run::<DiagonalEngine>(&demo_database, &demo_query);

    let file = File::open("data/uniprot_sprot.fasta").unwrap();
    let mut reader = FastaReader::new(BufReader::new(file));
    let database = reader.next().unwrap();
    let queries = reader.take(10_000).collect();

    let aligns1 = bench_sequential::<NaiveEngine>(&database, &queries);
    let aligns2 = bench_parallel::<NaiveEngine>(&database, &queries);
    assert!(aligns1 == aligns2);

    // bench_sequential::<DiagonalEngine>(&database, &queries);
    let aligns3 = bench_parallel::<DiagonalEngine>(&database, &queries);
    assert!(aligns1 == aligns3);
}
