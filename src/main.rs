mod engine;
mod fasta;
mod metrics;
mod model;

use std::{io::{BufReader, self, Write}, fs::File, sync::{Mutex, Arc}};
use rayon::prelude::*;

use engine::{NaiveEngine, Engine};
use fasta::FastaReader;
use metrics::Metrics;
use model::Sequence;

fn run<E>(database: &Sequence, query: &Sequence) where E: Default + Engine {
    let engine = E::default();
    println!("=== {} ===", E::name());

    let aligned = engine.align(database, query, &Arc::new(Mutex::new(Metrics::new())));
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);
}

fn bench_sequential<E>(database: &Sequence, queries: &Vec<Sequence>) where E: Default + Engine {
    let engine = E::default();
    println!("=== {} (sequential) ===", E::name());

    let total = queries.len();
    let metrics = Arc::new(Mutex::new(Metrics::new()));
    for (i, query) in queries.iter().enumerate() {
        engine.align(database, query, &metrics);
        if i % 100 == 0 {
            print!("\r[{} %]", (i * 100) / total);
            io::stdout().flush().unwrap();
        }
    }

    print!("\r");
    metrics.lock().unwrap().print();
}

fn bench_parallel<E>(database: &Sequence, queries: &Vec<Sequence>) where E: Default + Engine + Sync {
    let engine = E::default();
    println!("=== {} (parallel) ===", E::name());

    let metrics = Arc::new(Mutex::new(Metrics::new()));
    queries.par_iter().for_each(|query| {
        engine.align(database, query, &metrics);
    });

    metrics.lock().unwrap().print();
}

fn main() {
    run::<NaiveEngine>(&"TGTTACGG".parse().unwrap(), &"GGTTGACTA".parse().unwrap());

    let file = File::open("data/uniprot_sprot.fasta").unwrap();
    let mut reader = FastaReader::new(BufReader::new(file));
    let database = reader.next().unwrap();
    let queries = reader.take(10_000).collect();

    bench_sequential::<NaiveEngine>(&database, &queries);
    bench_parallel::<NaiveEngine>(&database, &queries);
}
