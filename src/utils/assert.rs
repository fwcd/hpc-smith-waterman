use std::fmt::Debug;
use pretty_assertions::assert_eq;

/// A facility that asserts that all values it's given
/// are equal (more specificially, that every value
/// after the first is equal to the first).
pub struct EqualAsserter<T> {
    first: Option<T>,
}

impl<T> EqualAsserter<T> where T: Debug + Eq {
    pub fn new() -> Self {
        Self { first: None }
    }

    pub fn feed(&mut self, name: &str, value: T) {
        if let Some(ref first) = self.first {
            assert_eq!(first, &value, "{} did not produce the correct result", name.to_string());
        } else {
            self.first = Some(value);
        }
    }
}
