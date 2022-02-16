mod engine;
mod fasta;
mod model;

use std::{io::BufReader, fs::File};

use fasta::FastaReader;

fn main() {
    let file = File::open("data/uniprot_sprot.fasta").unwrap();
    let reader = FastaReader::new(BufReader::new(file));

    for seq in reader {
        println!("Got {} of length {}", seq.name, seq.raw.len());
    }
}
