mod engine;
mod fasta;
mod model;

use std::{io::{BufReader, self, Write}, fs::File, time::Instant};
use rayon::prelude::*;

use engine::{NaiveEngine, Engine};
use fasta::FastaReader;
use model::Sequence;

fn run<E>(database: &Sequence, query: &Sequence) where E: Default + Engine {
    let engine = E::default();
    println!("=== {} ===", E::name());

    let aligned = engine.align(database, query);
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);
}

fn bench_sequential<E>(database: &Sequence, queries: &Vec<Sequence>) where E: Default + Engine {
    let engine = E::default();
    println!("=== {} (sequential) ===", E::name());

    let total = queries.len();
    let start = Instant::now();
    for (i, query) in queries.iter().enumerate() {
        engine.align(database, query);
        if i % 100 == 0 {
            print!("\r[{} %]", (i * 100) / total);
            io::stdout().flush().unwrap();
        }
    }

    let elapsed_ms = start.elapsed().as_millis();
    println!("\r{} ms for {} sequences", elapsed_ms, total)
}

fn bench_parallel<E>(database: &Sequence, queries: &Vec<Sequence>) where E: Default + Engine + Sync {
    let engine = E::default();
    println!("=== {} (parallel) ===", E::name());

    let total = queries.len();
    let start = Instant::now();
    queries.par_iter().for_each(|query| {
        engine.align(database, query);
    });

    let elapsed_ms = start.elapsed().as_millis();
    println!("\r{} ms for {} sequences", elapsed_ms, total)
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
