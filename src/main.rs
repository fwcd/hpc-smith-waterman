use std::{io::BufReader, fs::File};

use fasta::FastaReader;

mod fasta;
mod model;

fn main() {
    let file = File::open("data/uniprot_sprot.fasta").unwrap();
    let reader = FastaReader::new(BufReader::new(file));

    for seq in reader {
        println!("Got {} with {:?}", seq.name, seq.raw);
    }
}
