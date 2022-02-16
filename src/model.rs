use std::{fmt, ops::Index, str::FromStr};

/// A (named) nucleid sequence.
#[derive(PartialEq, Eq)]
pub struct Sequence {
    pub name: String,
    pub raw: Vec<u8>,
}

/// An alignment on a nucleid sequence.
#[derive(PartialEq, Eq)]
pub struct AlignedSequence<'a> {
    pub sequence: &'a Sequence,
    pub indices: Vec<usize>,
}

/// An alignment of two sequences.
#[derive(PartialEq, Eq)]
pub struct AlignedPair<'a> {
    pub database: AlignedSequence<'a>,
    pub query: AlignedSequence<'a>,
}

impl Sequence {
    pub fn new(name: &str, raw: Vec<u8>) -> Self {
        Self { name: name.to_owned(), raw }
    }

    /// The length of the sequence.
    pub fn len(&self) -> usize {
        self.raw.len()
    }
}

impl FromStr for Sequence {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new("Parsed", s.as_bytes().iter().map(|&x| x).collect()))
    }
}

impl Index<usize> for Sequence {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.raw[index]
    }
}

impl<'a> AlignedSequence<'a> {
    pub fn new(sequence: &'a Sequence, indices: Vec<usize>) -> Self {
        Self { sequence, indices }
    }
}

impl<'a> AlignedPair<'a> {
    pub fn new(database: AlignedSequence<'a>, query: AlignedSequence<'a>) -> Self {
        Self { database, query }
    }
}

impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8(self.raw.clone()).unwrap())
    }
}

impl<'a> fmt::Display for AlignedSequence<'a> {
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
