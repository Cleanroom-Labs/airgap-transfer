/// Unpack command — reconstruct files from tar chunks.
use colored::Colorize;

use crate::UnpackArgs;
use crate::chunker;
use crate::error::{AirgapError, Result};
use crate::manifest::{MANIFEST_FILENAME, Manifest};
use crate::progress::{TransferProgress, format_bytes};
use crate::usb;
use crate::verifier;

/// Execute the unpack operation.
pub fn run(args: &UnpackArgs) -> Result<()> {
    let source = &args.source;
    let dest = &args.dest;

    // Overwrite protection: warn if destination is non-empty
    if dest.exists() && dest.read_dir()?.next().is_some() && !args.force {
        return Err(AirgapError::UserAbort(format!(
            "destination {} is not empty. Use --force to overwrite.",
            dest.display()
        )));
    }

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

    // Extract chunks
    println!(
        "{} Unpacking {} to {}...",
        "→".green().bold(),
        format_bytes(manifest.total_size_bytes),
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

#[cfg(test)]
mod tests {
    use crate::UnpackArgs;
    use crate::error::AirgapError;

    /// TC-SAF-001: Unpack aborts when destination is non-empty.
    #[test]
    fn unpack_aborts_when_dest_not_empty() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // Create a file in dest to make it non-empty
        std::fs::write(dest_dir.path().join("existing.txt"), b"content").unwrap();

        // Create a minimal manifest in source so the command gets past loading
        // (it checks dest before loading manifest, so this is needed)
        let args = UnpackArgs {
            source: source_dir.path().to_path_buf(),
            dest: dest_dir.path().to_path_buf(),
            no_verify: false,
            keep_chunks: false,
            force: false,
            verbose: false,
        };

        let result = super::run(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            AirgapError::UserAbort(msg) => {
                assert!(msg.contains("--force"), "error should suggest --force");
                assert!(msg.contains("not empty"));
            }
            other => panic!("expected UserAbort, got: {other}"),
        }
    }
}
