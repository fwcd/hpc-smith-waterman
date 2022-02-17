use std::{borrow::Borrow, io::Write, fmt::Debug};

/// Creates an ASCII-art-esque box around a string.
pub fn pretty_box(s: impl Borrow<str>) -> String {
    let gutter = "─".repeat(s.borrow().len() + 2);
    format!("┌{}┐\n│ {} │\n└{}┘", gutter, s.borrow(), gutter)
}

/// Formats a matrix of the given dimensions.
pub fn pretty_matrix<T>(mat: &[T], width: usize) -> String where T: Debug {
    let mut s = Vec::new();
    for i in (0..mat.len()).step_by(width) {
        writeln!(&mut s, "{:?}", &mat[i..(i + width)]).unwrap();
    }
    String::from_utf8(s).unwrap()
}
