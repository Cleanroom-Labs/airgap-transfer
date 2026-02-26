/// Pack command — split source files into tar chunks with integrity verification.
use std::path::Path;

use colored::Colorize;

use crate::PackArgs;
use crate::chunker;
use crate::error::{AirgapError, Result};
use crate::manifest::{MANIFEST_FILENAME, Manifest};
use crate::progress::{TransferProgress, format_bytes};
use crate::usb;
use crate::verifier;

/// Execute the pack operation.
pub fn run(args: &PackArgs) -> Result<()> {
    let source = &args.source;
    let dest = &args.dest;

    // Validate source exists
    if !source.exists() {
        return Err(AirgapError::InvalidPath(format!(
            "source does not exist: {}",
            source.display()
        )));
    }

    // Check for existing manifest (overwrite protection)
    let manifest_path = dest.join(MANIFEST_FILENAME);
    if manifest_path.exists() && !args.force {
        return Err(AirgapError::UserAbort(
            "destination already contains a manifest. Use --force to overwrite.".to_string(),
        ));
    }

    // Resolve the hash algorithm
    let algorithm = verifier::algorithm_from_name(&args.hash_algorithm)?;

    // Calculate total size
    let total_size = chunker::calculate_total_size(source)?;

    if args.dry_run {
        print_dry_run(
            source,
            dest,
            total_size,
            args.chunk_size,
            &args.hash_algorithm,
        );
        return Ok(());
    }

    // Check available space
    let dest_parent = if dest.exists() {
        dest.as_path()
    } else {
        dest.parent().unwrap_or(dest)
    };
    let available = usb::get_available_space(dest_parent)?;
    if available < total_size {
        return Err(AirgapError::InsufficientSpace {
            needed: total_size,
            available,
        });
    }

    // Create manifest
    let mut manifest = Manifest::new_pack(
        &source.to_string_lossy(),
        total_size,
        args.chunk_size,
        algorithm.name(),
    );

    // Pack with progress
    let progress = TransferProgress::new(total_size, args.verbose);

    println!(
        "{} Packing {} ({}) into chunks of {}...",
        "→".green().bold(),
        source.display(),
        format_bytes(total_size),
        format_bytes(args.chunk_size)
    );

    chunker::pack_to_chunks(
        source,
        dest,
        args.chunk_size,
        algorithm.as_ref(),
        &mut manifest,
        &progress,
    )?;

    progress.finish("packed");

    // Verify checksums if requested
    if !args.no_verify {
        println!("{} Verifying chunk checksums...", "→".green().bold());
        let verify_progress =
            TransferProgress::new_items(manifest.chunk_count as u64, args.verbose);
        for chunk in &manifest.chunks {
            let chunk_path = dest.join(&chunk.filename);
            let actual = verifier::compute_checksum(&chunk_path, algorithm.as_ref())?;
            if actual != chunk.checksum {
                return Err(AirgapError::Checksum {
                    path: chunk_path,
                    expected: chunk.checksum.clone(),
                    actual,
                });
            }
            verify_progress.advance(1);
        }
        verify_progress.finish("verified");
        println!("{} All checksums verified", "✓".green().bold());
    }

    // Save manifest
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
    println!("  Total size: {}", format_bytes(total_size));
    println!("  Chunk size: {}", format_bytes(chunk_size));
    println!("  Chunks:     {}", chunk_count);
    println!("  Algorithm:  {}", algorithm);
    for i in 0..chunk_count {
        println!("    chunk_{:03}.tar", i);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::PackArgs;
    use crate::error::AirgapError;
    use crate::manifest::{MANIFEST_FILENAME, Manifest};

    /// TC-SAF-001: Pack aborts when destination has existing manifest.
    #[test]
    fn pack_aborts_without_force() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // Create source file
        std::fs::write(src_dir.path().join("test.txt"), b"data").unwrap();

        // Create existing manifest at destination
        let mut manifest = Manifest::new_pack("/dummy", 100, 100, "sha256");
        manifest
            .save(&dest_dir.path().join(MANIFEST_FILENAME))
            .unwrap();

        let args = PackArgs {
            source: src_dir.path().join("test.txt"),
            dest: dest_dir.path().to_path_buf(),
            chunk_size: 1_000_000,
            hash_algorithm: "sha256".to_string(),
            dry_run: false,
            no_verify: false,
            force: false,
            verbose: false,
        };

        let result = super::run(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            AirgapError::UserAbort(msg) => {
                assert!(msg.contains("--force"), "error should suggest --force");
            }
            other => panic!("expected UserAbort, got: {other}"),
        }
    }

    /// TC-SAF-005: Pack succeeds with --force over existing manifest.
    #[test]
    fn pack_succeeds_with_force() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // Create source file
        std::fs::write(src_dir.path().join("test.txt"), b"data").unwrap();

        // Create existing manifest at destination
        let mut manifest = Manifest::new_pack("/dummy", 100, 100, "sha256");
        manifest
            .save(&dest_dir.path().join(MANIFEST_FILENAME))
            .unwrap();

        let args = PackArgs {
            source: src_dir.path().join("test.txt"),
            dest: dest_dir.path().to_path_buf(),
            chunk_size: 1_000_000,
            hash_algorithm: "sha256".to_string(),
            dry_run: false,
            no_verify: false,
            force: true,
            verbose: false,
        };

        assert!(super::run(&args).is_ok());
    }

    /// Pack aborts when source does not exist.
    #[test]
    fn pack_rejects_missing_source() {
        let dest_dir = tempfile::tempdir().unwrap();

        let args = PackArgs {
            source: PathBuf::from("/nonexistent/file"),
            dest: dest_dir.path().to_path_buf(),
            chunk_size: 1_000_000,
            hash_algorithm: "sha256".to_string(),
            dry_run: false,
            no_verify: false,
            force: false,
            verbose: false,
        };

        let result = super::run(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            AirgapError::InvalidPath(msg) => {
                assert!(msg.contains("does not exist"));
            }
            other => panic!("expected InvalidPath, got: {other}"),
        }
    }
}
