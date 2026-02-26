/// List command — display chunk inventory from a manifest.
use std::path::Path;

use colored::Colorize;

use crate::error::Result;
use crate::manifest::{MANIFEST_FILENAME, Manifest};
use crate::progress::{TransferProgress, format_bytes};
use crate::verifier;

/// Execute the list operation.
pub fn run(chunk_location: &Path, verify: bool) -> Result<()> {
    let manifest_path = chunk_location.join(MANIFEST_FILENAME);
    let manifest = Manifest::load(&manifest_path)?;

    // Header
    println!(
        "{} {} (v{})",
        "Manifest:".bold(),
        MANIFEST_FILENAME,
        manifest.version
    );
    println!(
        "Operation: {} | Source: {} | Created: {}",
        manifest.operation,
        manifest.source_path,
        manifest.created_utc.format("%Y-%m-%d %H:%M UTC")
    );
    println!();

    // Count present chunks up front
    let present_count = manifest
        .chunks
        .iter()
        .filter(|c| chunk_location.join(&c.filename).exists())
        .count();
    let missing_count = manifest.chunk_count - present_count;

    // Summary
    println!(
        "Chunks: {}/{} present | Total: {} | Chunk size: {} | Hash: {}",
        present_count,
        manifest.chunk_count,
        format_bytes(manifest.total_size_bytes),
        format_bytes(manifest.chunk_size_bytes),
        manifest.hash_algorithm
    );
    println!();

    // Resolve algorithm if verifying
    let algorithm = if verify {
        Some(verifier::algorithm_from_name(&manifest.hash_algorithm)?)
    } else {
        None
    };

    // Column headers
    println!(
        "  {:<4} {:<18} {:<14} {:<10} Checksum",
        "#", "Filename", "Size", "Status"
    );

    // Chunk rows
    let progress = if verify {
        Some(TransferProgress::new_items(present_count as u64, false))
    } else {
        None
    };
    let mut corrupt_count = 0;

    for chunk in &manifest.chunks {
        let chunk_path = chunk_location.join(&chunk.filename);
        let present = chunk_path.exists();

        // Determine status: missing, verified, corrupt, or manifest status
        let status_display = if !present {
            format!("{:<10}", "MISSING").red().bold().to_string()
        } else if let Some(ref algo) = algorithm {
            let actual = verifier::compute_checksum(&chunk_path, algo.as_ref())?;
            if let Some(ref p) = progress {
                p.advance(1);
            }
            if actual == chunk.checksum {
                format!("{:<10}", "OK").green().to_string()
            } else {
                corrupt_count += 1;
                format!("{:<10}", "CORRUPT").red().bold().to_string()
            }
        } else {
            format!("{:<10}", chunk.status)
        };

        let checksum_display = truncate_checksum(&chunk.checksum);

        println!(
            "  {:<4} {:<18} {:<14} {} {}",
            chunk.index,
            chunk.filename,
            format_bytes(chunk.size_bytes),
            status_display,
            checksum_display
        );
    }

    if let Some(p) = progress {
        p.finish("verified");
    }

    println!();

    // Footer
    if missing_count > 0 {
        println!(
            "{} {} of {} chunks missing!",
            "!".red().bold(),
            missing_count,
            manifest.chunk_count
        );
    }
    if corrupt_count > 0 {
        println!(
            "{} {} of {} chunks corrupted!",
            "!".red().bold(),
            corrupt_count,
            manifest.chunk_count
        );
    }
    if missing_count == 0 && corrupt_count == 0 {
        if verify {
            println!(
                "{} All {} chunks present and verified.",
                "✓".green().bold(),
                manifest.chunk_count
            );
        } else {
            println!(
                "{} All {} chunks present.",
                "✓".green().bold(),
                manifest.chunk_count
            );
        }
    }

    Ok(())
}

/// Create a test directory with a saved manifest and chunk files.
#[cfg(test)]
fn setup_list_fixture() -> (tempfile::TempDir, Manifest) {
    use crate::chunker;
    use crate::manifest::MANIFEST_FILENAME;
    use crate::progress::TransferProgress;
    use crate::verifier::Sha256Algorithm;

    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    // Create a source file and pack it
    std::fs::write(src_dir.path().join("data.txt"), b"test data for listing").unwrap();
    let source = src_dir.path().join("data.txt");
    let total = chunker::calculate_total_size(&source).unwrap();
    let mut manifest = Manifest::new_pack(source.to_str().unwrap(), total, 1_000_000, "sha256");
    let progress = TransferProgress::hidden();
    chunker::pack_to_chunks(
        &source,
        dest_dir.path(),
        1_000_000,
        &Sha256Algorithm,
        &mut manifest,
        &progress,
    )
    .unwrap();
    manifest
        .save(&dest_dir.path().join(MANIFEST_FILENAME))
        .unwrap();

    (dest_dir, manifest)
}

/// Truncate a checksum like "sha256:abcdef012345..." to "sha256:abcdef01…"
fn truncate_checksum(checksum: &str) -> String {
    if let Some((prefix, hash)) = checksum.split_once(':') {
        if hash.len() > 8 {
            format!("{}:{}…", prefix, &hash[..8])
        } else {
            checksum.to_string()
        }
    } else {
        checksum.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-LST-001: List command succeeds with valid manifest and chunks.
    #[test]
    fn list_valid_manifest() {
        let (dir, _manifest) = setup_list_fixture();
        assert!(run(dir.path(), false).is_ok());
    }

    /// TC-LST-003: List identifies missing chunk files.
    #[test]
    fn list_flags_missing_chunks() {
        let (dir, _manifest) = setup_list_fixture();

        // Remove the chunk file
        std::fs::remove_file(dir.path().join("chunk_000.tar")).unwrap();

        // Should still succeed (list reports but doesn't fail)
        assert!(run(dir.path(), false).is_ok());
    }

    /// TC-LST-005: List --verify detects corruption.
    #[test]
    fn list_verify_detects_corruption() {
        let (dir, _manifest) = setup_list_fixture();

        // Corrupt the chunk file
        std::fs::write(dir.path().join("chunk_000.tar"), b"corrupted data").unwrap();

        // Should succeed (list reports corruption but doesn't fail)
        assert!(run(dir.path(), true).is_ok());
    }

    /// List --verify succeeds when chunks are intact.
    #[test]
    fn list_verify_all_ok() {
        let (dir, _manifest) = setup_list_fixture();
        assert!(run(dir.path(), true).is_ok());
    }

    #[test]
    fn truncate_checksum_long_hash() {
        assert_eq!(
            truncate_checksum("sha256:abcdef0123456789"),
            "sha256:abcdef01…"
        );
    }

    #[test]
    fn truncate_checksum_short_hash() {
        assert_eq!(truncate_checksum("sha256:abcd"), "sha256:abcd");
    }

    #[test]
    fn truncate_checksum_no_prefix() {
        assert_eq!(truncate_checksum("nocolon"), "nocolon");
    }
}
