/// AirGap Transfer — large file transfer utility for air-gapped environments.
///
/// Splits data into chunks, tracks state via a JSON manifest, and verifies
/// integrity with pluggable cryptographic checksums.
mod chunker;
mod commands;
mod error;
mod manifest;
mod progress;
mod usb;
mod verifier;

use std::path::PathBuf;
use std::process;

use clap::{Args, Parser, Subcommand};

/// Large file transfer utility for air-gapped environments.
#[derive(Parser)]
#[command(name = "airgap-transfer", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Split source files into chunks for transfer across an air gap.
    Pack(PackArgs),

    /// Reconstruct files from chunks.
    Unpack(UnpackArgs),

    /// Display chunk inventory from a manifest.
    List {
        /// Directory containing the manifest and chunks.
        chunk_location: PathBuf,
    },
}

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

    /// Show detailed progress.
    #[arg(long, short)]
    pub verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Pack(args) => commands::pack::run(&args),
        Commands::Unpack(args) => commands::unpack::run(&args),
        Commands::List { chunk_location } => commands::list::run(&chunk_location),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

/// Parse a human-readable size string like "1GB", "500MB", "1024".
fn parse_size(s: &str) -> Result<u64, String> {
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

    #[test]
    fn parse_size_plain_number() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
    }

    #[test]
    fn parse_size_kb() {
        assert_eq!(parse_size("10KB").unwrap(), 10 * 1024);
    }

    #[test]
    fn parse_size_mb() {
        assert_eq!(parse_size("5MB").unwrap(), 5 * 1_048_576);
    }

    #[test]
    fn parse_size_gb() {
        assert_eq!(parse_size("1GB").unwrap(), 1_073_741_824);
    }

    #[test]
    fn parse_size_lowercase() {
        assert_eq!(parse_size("2gb").unwrap(), 2 * 1_073_741_824);
    }

    #[test]
    fn parse_size_invalid() {
        assert!(parse_size("abc").is_err());
    }
}
