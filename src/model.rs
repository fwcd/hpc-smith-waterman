use std::fmt;

/// A (named) nucleid sequence.
pub struct Sequence {
    pub name: String,
    pub raw: Vec<u8>,
}

/// An alignment on a nucleid sequence.
pub struct AlignedSequence {
    pub sequence: Sequence,
    pub indices: Vec<usize>,
}

/// An alignment of two sequences.
pub struct AlignedPair {
    pub left: AlignedSequence,
    pub right: AlignedSequence,
}

impl Sequence {
    pub fn new(name: &str, raw: Vec<u8>) -> Self {
        Self { name: name.to_owned(), raw }
    }
}

impl AlignedSequence {
    pub fn new(sequence: Sequence, indices: Vec<usize>) -> Self {
        Self { sequence, indices }
    }
}

impl AlignedPair {
    pub fn new(left: AlignedSequence, right: AlignedSequence) -> Self {
        Self { left, right }
    }
}

impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8(self.raw.clone()).unwrap())
    }
}

impl fmt::Display for AlignedSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut last: Option<usize> = None;
        for &i in &self.indices {
            let c = self.sequence.raw[i] as char;
            if last == Some(i) {
                write!(f, "-")?;
            } else {
                write!(f, "{}", c)?;
            }
            last = Some(i);
        }
        Ok(())
    }
}
