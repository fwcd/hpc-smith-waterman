mod engine;
mod fasta;
mod model;

use std::{io::BufReader, fs::File};

use engine::{NaiveEngine, Engine};
use fasta::FastaReader;
use model::Sequence;

fn run_algorithm<E>(database: Sequence, query: Sequence) where E: Default + Engine {
    let engine = E::default();
    let aligned = engine.align(database, query);

    println!("=== {} ===", E::name());
    println!("D: {}", aligned.database);
    println!("Q: {}", aligned.query);
}

fn main() {
    run_algorithm::<NaiveEngine>("TGTTACGG".parse().unwrap(), "GGTTGACTA".parse().unwrap());

    // let file = File::open("data/uniprot_sprot.fasta").unwrap();
    // let reader = FastaReader::new(BufReader::new(file));

    // for seq in reader {
    //     println!("Got {} of length {}", seq.name, seq.raw.len());
    // }
}
