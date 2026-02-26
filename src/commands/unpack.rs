/// Unpack command — reconstruct files from tar chunks.
use colored::Colorize;

use crate::UnpackArgs;
use crate::chunker;
use crate::error::{AirgapError, Result};
use crate::manifest::{MANIFEST_FILENAME, Manifest};
use crate::progress::TransferProgress;
use crate::usb;
use crate::verifier;

/// Execute the unpack operation.
pub fn run(args: &UnpackArgs) -> Result<()> {
    let source = &args.source;
    let dest = &args.dest;

    // Load manifest
    let manifest_path = source.join(MANIFEST_FILENAME);
    let manifest = Manifest::load(&manifest_path)?;

    // Resolve hash algorithm from manifest
    let algorithm = verifier::algorithm_from_name(&manifest.hash_algorithm)?;

    // Validate chunk completeness
    println!(
        "{} Checking {} chunks...",
        "→".green().bold(),
        manifest.chunk_count
    );
    for chunk in &manifest.chunks {
        let chunk_path = source.join(&chunk.filename);
        if !chunk_path.exists() {
            return Err(AirgapError::ChunkMissing(format!(
                "{} not found in {}",
                chunk.filename,
                source.display()
            )));
        }
    }

    // Verify checksums before extraction
    if !args.no_verify {
        println!("{} Verifying chunk checksums...", "→".green().bold());
        let verify_progress =
            TransferProgress::new_items(manifest.chunk_count as u64, args.verbose);
        for chunk in &manifest.chunks {
            let chunk_path = source.join(&chunk.filename);
            let valid =
                verifier::verify_checksum(&chunk_path, &chunk.checksum, algorithm.as_ref())?;
            if !valid {
                return Err(AirgapError::Checksum {
                    path: chunk_path,
                    expected: chunk.checksum.clone(),
                    actual: "mismatch".to_string(),
                });
            }
            verify_progress.advance(1);
        }
        verify_progress.finish("verified");
        println!("{} All checksums verified", "✓".green().bold());
    }

    // Extract chunks
    println!(
        "{} Unpacking {} bytes to {}...",
        "→".green().bold(),
        manifest.total_size_bytes,
        dest.display()
    );
    let progress = TransferProgress::new(manifest.total_size_bytes, args.verbose);

    chunker::unpack_from_chunks(source, dest, &manifest, &progress)?;

    progress.finish("unpacked");

    // Sync filesystem
    usb::sync_filesystem()?;

    println!(
        "{} Unpack complete: {} chunks extracted to {}",
        "✓".green().bold(),
        manifest.chunk_count,
        dest.display()
    );

    // Clean up chunks unless --keep-chunks
    if !args.keep_chunks {
        for chunk in &manifest.chunks {
            let chunk_path = source.join(&chunk.filename);
            std::fs::remove_file(&chunk_path)?;
        }
        std::fs::remove_file(&manifest_path)?;
        println!("{} Cleaned up chunks and manifest", "✓".green().bold());
    }

    Ok(())
}
