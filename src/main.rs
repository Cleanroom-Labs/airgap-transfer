/// AirGap Transfer — large file transfer utility for air-gapped environments.
///
/// Splits data into chunks, tracks state via a JSON manifest, and verifies
/// integrity with pluggable cryptographic checksums.
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use airgap_transfer::{PackArgs, UnpackArgs, commands};

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

        /// Compute checksums for present chunks and verify against manifest.
        #[arg(long)]
        verify: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Pack(args) => commands::pack::run(&args),
        Commands::Unpack(args) => commands::unpack::run(&args),
        Commands::List {
            chunk_location,
            verify,
        } => commands::list::run(&chunk_location, verify),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
