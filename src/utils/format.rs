use std::borrow::Borrow;

/// Creates an ASCII-art-esque box around a string.
pub fn pretty_box(s: impl Borrow<str>) -> String {
    let gutter = "─".repeat(s.borrow().len() + 2);
    format!("┌{}┐\n│ {} │\n└{}┘", gutter, s.borrow(), gutter)
}
