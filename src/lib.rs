//! AirGap Transfer — core library for chunked file transfer across air-gapped environments.
//!
//! This crate provides the building blocks for splitting files into chunks,
//! verifying integrity with pluggable cryptographic checksums, and managing
//! transfer state via a JSON manifest. It is designed for air-gapped
//! environments where data must be moved on removable media (USB drives).
//!
//! # Modules
//!
//! - [`chunker`] — Streaming chunk creation and reconstruction (tar format)
//! - [`commands`] — Pack, unpack, and list command implementations
//! - [`error`] — Error types and result aliases
//! - [`manifest`] — JSON manifest for metadata and state persistence
//! - [`progress`] — Progress bar and byte formatting utilities
//! - [`prompt`] — Interactive user prompts for USB swapping
//! - [`usb`] — USB drive detection and capacity checks (platform-specific)
//! - [`verifier`] — Pluggable hash verification (`HashAlgorithm` trait)

pub mod chunker;
pub mod commands;
pub mod error;
pub mod manifest;
pub mod progress;
pub mod prompt;
pub mod usb;
pub mod verifier;

use std::path::PathBuf;

use clap::Args;

/// Arguments for the pack subcommand.
#[derive(Args)]
pub struct PackArgs {
    /// Source file or directory to pack.
    pub source: PathBuf,

    /// Destination directory for chunks and manifest.
    pub dest: PathBuf,

    /// Target size per chunk (bytes). Accepts suffixes: KB, MB, GB.
    #[arg(long, default_value = "1073741824", value_parser = parse_size)]
    pub chunk_size: u64,

    /// Hash algorithm for chunk verification.
    #[arg(long, default_value = "sha256")]
    pub hash_algorithm: String,

    /// Preview the operation without writing any files.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip checksum verification after packing.
    #[arg(long)]
    pub no_verify: bool,

    /// Overwrite existing chunks and manifest at destination.
    #[arg(long)]
    pub force: bool,

    /// Resume an interrupted pack operation.
    #[arg(long)]
    pub resume: bool,

    /// Show detailed progress for each chunk.
    #[arg(long, short)]
    pub verbose: bool,
}

/// Arguments for the unpack subcommand.
#[derive(Args)]
pub struct UnpackArgs {
    /// Directory containing chunks and manifest.
    pub source: PathBuf,

    /// Destination directory for reconstructed files.
    pub dest: PathBuf,

    /// Skip checksum verification during unpack.
    #[arg(long)]
    pub no_verify: bool,

    /// Keep chunk files after successful unpack.
    #[arg(long)]
    pub keep_chunks: bool,

    /// Overwrite existing files at destination.
    #[arg(long)]
    pub force: bool,

    /// Resume an interrupted unpack operation.
    #[arg(long)]
    pub resume: bool,

    /// Show detailed progress.
    #[arg(long, short)]
    pub verbose: bool,
}

/// Parse a human-readable size string like "1GB", "500MB", "1024".
pub fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if let Ok(n) = s.parse::<u64>() {
        return Ok(n);
    }

    let s_upper = s.to_uppercase();
    let (num_part, multiplier) = if let Some(n) = s_upper.strip_suffix("GB") {
        (n, 1_073_741_824u64)
    } else if let Some(n) = s_upper.strip_suffix("MB") {
        (n, 1_048_576u64)
    } else if let Some(n) = s_upper.strip_suffix("KB") {
        (n, 1_024u64)
    } else {
        return Err(format!(
            "invalid size: {s}. Use a number with optional KB, MB, or GB suffix"
        ));
    };

    let n: u64 = num_part
        .trim()
        .parse()
        .map_err(|_| format!("invalid size number: {num_part}"))?;
    Ok(n * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec: TC-TRANSFER-CLI-006
    #[test]
    fn parse_size_plain_number() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
    }

    /// Spec: TC-TRANSFER-CLI-006
    #[test]
    fn parse_size_kb() {
        assert_eq!(parse_size("10KB").unwrap(), 10 * 1024);
    }

    /// Spec: TC-TRANSFER-CLI-006
    #[test]
    fn parse_size_mb() {
        assert_eq!(parse_size("5MB").unwrap(), 5 * 1_048_576);
    }

    /// Spec: TC-TRANSFER-CLI-006
    #[test]
    fn parse_size_gb() {
        assert_eq!(parse_size("1GB").unwrap(), 1_073_741_824);
    }

    /// Spec: TC-TRANSFER-CLI-006
    #[test]
    fn parse_size_lowercase() {
        assert_eq!(parse_size("2gb").unwrap(), 2 * 1_073_741_824);
    }

    /// Spec: TC-TRANSFER-CLI-006
    #[test]
    fn parse_size_invalid() {
        assert!(parse_size("abc").is_err());
    }
}
