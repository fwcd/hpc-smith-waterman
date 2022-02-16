mod engine;
mod fasta;
mod model;

use std::{io::BufReader, fs::File};

use engine::{NaiveEngine, Engine};
use fasta::FastaReader;
use model::Sequence;

fn main() {
    let d = Sequence::new("D", b"GGTTGACTA".iter().cloned().collect());
    let q = Sequence::new("Q", b"TGTTACGG".iter().cloned().collect());

    let engine = NaiveEngine::default();
    let alignment = engine.align(d, q);
    println!("D: {}", alignment.database);
    println!("Q: {}", alignment.query);

    // let file = File::open("data/uniprot_sprot.fasta").unwrap();
    // let reader = FastaReader::new(BufReader::new(file));

    // for seq in reader {
    //     println!("Got {} of length {}", seq.name, seq.raw.len());
    // }
}
