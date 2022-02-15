use std::io::BufRead;

use crate::model::Sequence;

const PREFIX: char = '>';

/// An abstraction for reading nucleid sequences in the FASTA
/// format from an underlying reader (e.g. a file).
pub struct FastaReader<R> {
    reader: R,
    line_index: usize,
    buffer: String,
    done: bool,
}

impl<R> FastaReader<R> where R: BufRead {
    pub fn new(reader: R) -> Self {
        let mut new = Self { reader, line_index: 0, buffer: String::with_capacity(128), done: false };
        new.next_line();
        new
    }

    fn current_line(&mut self) -> Option<&str> {
        if self.done {
            None
        } else {
            Some(self.buffer.as_str().trim())
        }
    }

    fn next_line(&mut self) -> Option<&str> {
        if self.done {
            return None;
        }
        self.buffer.clear();
        let byte_count = self.reader.read_line(&mut self.buffer).expect("Could not read line from FASTA file.");
        if byte_count == 0 {
            self.done = true;
        }
        self.current_line()
    }
}

impl<R> Iterator for FastaReader<R> where R: BufRead {
    type Item = Sequence;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = self.current_line()?;
        let mut raw: Vec<u8> = Vec::with_capacity(64);

        if let Some(name) = line.strip_prefix(PREFIX).map(|s| s.to_owned()) {
            loop {
                line = self.next_line()?;
                if line.starts_with(PREFIX) {
                    break Some(Sequence::new(name.as_str(), raw));
                } else {
                    raw.extend(line.as_bytes());
                }
            }
        } else {
            panic!("Misformatted FASTA file, line does not begin with {} (at line {})", PREFIX, self.line_index);
        }
    }
}
