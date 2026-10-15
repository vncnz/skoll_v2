use std::time::Instant;
use colored::Colorize;

pub struct TimeOfScope {
    label: String,
    start: Instant,
}

impl TimeOfScope {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for TimeOfScope {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        eprintln!("{}", format!("{} {}ms", format!("{} time:", self.label).red(), elapsed.as_millis()).red());
    }
}