/// List command — display chunk inventory from a manifest.
use std::path::Path;

use colored::Colorize;

use crate::error::Result;
use crate::manifest::{MANIFEST_FILENAME, Manifest};
use crate::progress::format_bytes;

/// Execute the list operation.
pub fn run(chunk_location: &Path) -> Result<()> {
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
        format!("{:?}", manifest.operation).to_lowercase(),
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

    // Column headers
    println!(
        "  {:<4} {:<18} {:<14} {:<10} Checksum",
        "#", "Filename", "Size", "Status"
    );

    // Chunk rows
    for chunk in &manifest.chunks {
        let present = chunk_location.join(&chunk.filename).exists();

        // Pad before colorizing so ANSI escape codes don't break alignment
        let status_display = if !present {
            format!("{:<10}", "MISSING").red().bold().to_string()
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

    println!();

    // Footer
    if missing_count > 0 {
        println!(
            "{} {} of {} chunks missing!",
            "!".red().bold(),
            missing_count,
            manifest.chunk_count
        );
    } else {
        println!(
            "{} All {} chunks present.",
            "✓".green().bold(),
            manifest.chunk_count
        );
    }

    Ok(())
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
