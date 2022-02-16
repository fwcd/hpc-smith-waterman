mod engine;
mod fasta;
mod model;

use std::{io::BufReader, fs::File, time::Instant};

use engine::{NaiveEngine, Engine};
use fasta::FastaReader;
use model::Sequence;

fn run_algorithm<E>(database: &Sequence, query: &Sequence) where E: Default + Engine {
    let engine = E::default();
    println!("=== {} ===", E::name());

    let start = Instant::now();
    let aligned = engine.align(database, query);
    let delta_ms = start.elapsed().as_millis();
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);
    println!("(in {} ms)", delta_ms)
}

fn main() {
    run_algorithm::<NaiveEngine>(&"TGTTACGG".parse().unwrap(), &"GGTTGACTA".parse().unwrap());

    let file = File::open("data/uniprot_sprot.fasta").unwrap();
    let mut reader = FastaReader::new(BufReader::new(file));

    let database = reader.next().unwrap();
    for query in reader {
        run_algorithm::<NaiveEngine>(&database, &query);
    }
}
