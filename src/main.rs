mod engine;
mod fasta;
mod model;

use std::{io::{BufReader, self, Write}, fs::File, time::Instant};

use engine::{NaiveEngine, Engine};
use fasta::FastaReader;
use model::Sequence;

fn run_algorithm<E>(database: &Sequence, query: &Sequence) where E: Default + Engine {
    let engine = E::default();
    println!("=== {} ===", E::name());

    let aligned = engine.align(database, query);
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);
}

fn bench_algorithm<E>(database: &Sequence, queries: &Vec<Sequence>) where E: Default + Engine {
    let engine = E::default();
    println!("=== {} ===", E::name());

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

fn main() {
    run_algorithm::<NaiveEngine>(&"TGTTACGG".parse().unwrap(), &"GGTTGACTA".parse().unwrap());

    let file = File::open("data/uniprot_sprot.fasta").unwrap();
    let mut reader = FastaReader::new(BufReader::new(file));
    let database = reader.next().unwrap();
    let queries = reader.take(10_000).collect();

    bench_algorithm::<NaiveEngine>(&database, &queries);
}
