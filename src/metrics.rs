use std::time::{Instant, Duration};

/// A tool for tracking various metrics about the execution.
pub struct Metrics {
    start: Instant,
    cell_updates: usize,
    sequence_pairs: usize,
}

impl Metrics {
    /// Creates a new set of metrics.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            cell_updates: 0,
            sequence_pairs: 0,
        }
    }

    /// Records that a sequence pair has been processed.
    pub fn record_sequence_pair(&mut self) {
        self.sequence_pairs += 1;
    }

    /// Records that cell updates have been processed.
    pub fn record_cell_updates(&mut self, count: usize) {
        self.cell_updates += count;
    }

    /// Fetches the elapsed time.
    pub fn elapsed(&self) -> Duration { self.start.elapsed() }

    /// Prints the metrics.
    pub fn print(&self) {
        let elapsed = self.elapsed();
        println!("Elapsed: {:.2}s", elapsed.as_secs_f64());
        println!("Giga-CUPS: {:.2}", self.cell_updates as f64 / (1_000_000_000f64 * elapsed.as_secs_f64()));
        println!("Pairs: {:.2}", self.sequence_pairs);
    }
}
