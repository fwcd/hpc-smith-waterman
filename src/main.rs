mod engine;
mod fasta;
mod model;

use std::{io::BufReader, fs::File, time::Instant, borrow::Borrow};

use engine::{NaiveEngine, Engine};
use fasta::FastaReader;
use model::Sequence;

fn bench_algorithm<'a, E, I>(database: &'a Sequence, queries: I)
    where E: Default + Engine,
          I: IntoIterator,
          I::Item: Borrow<Sequence> {
    let engine = E::default();
    println!("=== {} ===", E::name());

    let start = Instant::now();
    for query in queries {
        engine.align(database, query.borrow());
    }

    let elapsed_ms = start.elapsed().as_millis();
    println!("(in {} ms)", elapsed_ms)
}

fn run_algorithm<E>(database: &Sequence, query: &Sequence)
    where E: Default + Engine {
    let engine = E::default();
    println!("=== {} ===", E::name());

    let aligned = engine.align(database, query);
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);
}

fn main() {
    run_algorithm::<NaiveEngine>(&"TGTTACGG".parse().unwrap(), &"GGTTGACTA".parse().unwrap());

    let file = File::open("data/uniprot_sprot.fasta").unwrap();
    let mut reader = FastaReader::new(BufReader::new(file));

    let database = reader.next().unwrap();
    bench_algorithm::<NaiveEngine, _>(&database, reader);
}
