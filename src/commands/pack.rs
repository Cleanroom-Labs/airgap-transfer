/// Pack command — split source files into tar chunks with integrity verification.
use std::path::Path;

use colored::Colorize;

use crate::chunker;
use crate::error::Result;
use crate::manifest::{MANIFEST_FILENAME, Manifest};
use crate::progress::TransferProgress;
use crate::usb;
use crate::verifier;

/// Execute the pack operation.
pub fn run(
    source: &Path,
    dest: &Path,
    chunk_size: u64,
    hash_algorithm: &str,
    dry_run: bool,
    no_verify: bool,
    verbose: bool,
) -> Result<()> {
    // Resolve the hash algorithm
    let algorithm = verifier::algorithm_from_name(hash_algorithm)?;

    // Calculate total size
    let total_size = chunker::calculate_total_size(source)?;

    if dry_run {
        print_dry_run(source, dest, total_size, chunk_size, hash_algorithm);
        return Ok(());
    }

    // Check available space
    let dest_parent = if dest.exists() {
        dest
    } else {
        dest.parent().unwrap_or(dest)
    };
    let available = usb::get_available_space(dest_parent)?;
    if available < total_size {
        return Err(crate::error::AirgapError::InsufficientSpace {
            needed: total_size,
            available,
        });
    }

    // Create manifest
    let mut manifest = Manifest::new_pack(
        &source.to_string_lossy(),
        total_size,
        chunk_size,
        algorithm.name(),
    );

    // Pack with progress
    let progress = TransferProgress::new(total_size, verbose);

    println!(
        "{} Packing {} ({} bytes) into chunks of {} bytes...",
        "→".green().bold(),
        source.display(),
        total_size,
        chunk_size
    );

    chunker::pack_to_chunks(
        source,
        dest,
        chunk_size,
        algorithm.as_ref(),
        &mut manifest,
        &progress,
    )?;

    progress.finish("packed");

    // Verify checksums if requested
    if !no_verify {
        println!("{} Verifying chunk checksums...", "→".green().bold());
        let verify_progress = TransferProgress::new_items(manifest.chunk_count as u64, verbose);
        for chunk in &manifest.chunks {
            let chunk_path = dest.join(&chunk.filename);
            let valid =
                verifier::verify_checksum(&chunk_path, &chunk.checksum, algorithm.as_ref())?;
            if !valid {
                return Err(crate::error::AirgapError::Checksum {
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

    // Save manifest
    let manifest_path = dest.join(MANIFEST_FILENAME);
    manifest.save(&manifest_path)?;
    println!(
        "{} Manifest saved to {}",
        "✓".green().bold(),
        manifest_path.display()
    );

    // Sync filesystem
    usb::sync_filesystem()?;

    println!(
        "{} Pack complete: {} chunks written to {}",
        "✓".green().bold(),
        manifest.chunk_count,
        dest.display()
    );

    Ok(())
}

fn print_dry_run(source: &Path, dest: &Path, total_size: u64, chunk_size: u64, algorithm: &str) {
    let chunk_count = if total_size == 0 {
        1
    } else {
        total_size.div_ceil(chunk_size) as usize
    };

    println!("{} Dry run — no files will be written", "ℹ".blue().bold());
    println!("  Source:     {}", source.display());
    println!("  Dest:       {}", dest.display());
    println!("  Total size: {} bytes", total_size);
    println!("  Chunk size: {} bytes", chunk_size);
    println!("  Chunks:     {}", chunk_count);
    println!("  Algorithm:  {}", algorithm);
    for i in 0..chunk_count {
        println!("    chunk_{:03}.tar", i);
    }
}
