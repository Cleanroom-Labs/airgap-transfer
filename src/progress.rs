/// Progress display for transfer operations.
///
/// Wraps [`indicatif::ProgressBar`] to provide consistent formatting
/// across pack, unpack, and verification stages.
use indicatif::{ProgressBar, ProgressStyle};

/// Configurable progress display for streaming operations.
pub struct TransferProgress {
    bar: ProgressBar,
    verbose: bool,
}

impl TransferProgress {
    /// Create a progress bar for a known total byte count.
    pub fn new(total_bytes: u64, verbose: bool) -> Self {
        let bar = ProgressBar::new(total_bytes);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("valid template")
                .progress_chars("#>-"),
        );
        Self { bar, verbose }
    }

    /// Create a progress bar for a known number of items (e.g. chunks).
    pub fn new_items(total_items: u64, verbose: bool) -> Self {
        let bar = ProgressBar::new(total_items);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} chunks ({eta})")
                .expect("valid template")
                .progress_chars("#>-"),
        );
        Self { bar, verbose }
    }

    /// Advance the progress bar by `n` units.
    pub fn advance(&self, n: u64) {
        self.bar.inc(n);
    }

    /// Print a message above the progress bar (only in verbose mode).
    pub fn verbose_message(&self, msg: &str) {
        if self.verbose {
            self.bar.println(msg);
        }
    }

    /// Mark the progress bar as finished with a message.
    pub fn finish(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }

    /// Create a hidden (no-op) progress bar for quiet operations.
    #[allow(dead_code)] // Available for non-interactive contexts in later phases.
    pub fn hidden() -> Self {
        let bar = ProgressBar::hidden();
        Self {
            bar,
            verbose: false,
        }
    }
}

/// Format a byte count as a human-readable string (e.g. "1.50 GB").
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_048_576;
    const GB: u64 = 1_073_741_824;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_units() {
        assert_eq!(format_bytes(0), "0 bytes");
        assert_eq!(format_bytes(512), "512 bytes");
        assert_eq!(format_bytes(1_024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
        assert_eq!(format_bytes(1_610_612_736), "1.50 GB");
    }
}
