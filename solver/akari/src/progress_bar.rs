use std::io::{Write, stdout};

/// Simple terminal progress bar for enumerating constraint completions.
pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
}

impl ProgressBar {
    /// Create progress bar with fixed total steps.
    pub fn new(total: usize) -> Self {
        let bar = Self {
            total,
            current: 0,
            width: 40,
        };
        if total != 0 {
            bar.render();
        }
        bar
    }

    /// Advance by one step and render.
    pub fn tick(&mut self) {
        if self.total == 0 {
            return;
        }
        self.current = (self.current + 1).min(self.total);
        self.render();
    }

    /// Finish the bar and move to the next line.
    pub fn finish(&mut self) {
        if self.total == 0 {
            return;
        }
        self.current = self.total;
        self.render();
        println!();
    }

    fn render(&self) {
        if self.total == 0 {
            return;
        }
        let ratio = self.current as f32 / self.total as f32;
        let filled = ((ratio * self.width as f32).round() as usize).min(self.width);
        let empty = self.width - filled;
        let bar = format!("{}{}", "=".repeat(filled), " ".repeat(empty));
        print!("\rProgress: [{bar}] {}/{}", self.current, self.total);
        let _ = stdout().flush();
    }
}
