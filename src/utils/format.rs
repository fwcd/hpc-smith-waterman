/// Creates an ASCII-art-esque box around a string.
pub fn pretty_box(s: &str) -> String {
    let gutter = "─".repeat(s.len() + 2);
    format!("┌{}┐\n│ {} │\n└{}┘", gutter, s, gutter)
}
