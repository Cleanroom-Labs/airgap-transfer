/// Interactive prompts for multi-USB workflows.
///
/// Provides stdin-based prompting for USB drive swapping during pack/unpack
/// operations that span multiple removable drives.
use std::io::{self, BufRead, Write};
use std::path::Path;

use colored::Colorize;

use crate::error::Result;
use crate::usb;

/// Prompt the user to swap USB drives, then wait for confirmation.
///
/// Displays a message indicating which chunk needs more space (or which
/// chunk is needed), syncs the filesystem, then waits for the user to
/// press Enter after inserting a new drive.
pub fn prompt_usb_swap(message: &str) -> Result<()> {
    // Sync before prompting for removal
    usb::sync_filesystem()?;

    println!();
    println!("{} {}", "⏏".yellow().bold(), message);
    print!("  Press Enter when ready...");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    Ok(())
}

/// Prompt for USB swap during pack when destination runs out of space.
///
/// Returns `Ok(())` once the user confirms and the destination has enough
/// space for at least `needed_bytes`.
pub fn prompt_pack_swap(dest: &Path, chunk_index: usize, needed_bytes: u64) -> Result<()> {
    loop {
        prompt_usb_swap(&format!(
            "Insufficient space for chunk_{:03}.tar. \
             Please insert a new USB drive at {} and remove the current one.",
            chunk_index,
            dest.display()
        ))?;

        let available = usb::get_available_space(dest)?;
        if available >= needed_bytes {
            println!(
                "  {} Drive detected with sufficient space.",
                "✓".green().bold()
            );
            return Ok(());
        }

        println!(
            "  {} Still insufficient space ({} available, {} needed). Try again.",
            "!".red().bold(),
            crate::progress::format_bytes(available),
            crate::progress::format_bytes(needed_bytes),
        );
    }
}

/// Prompt for USB swap during unpack when a chunk file is missing.
///
/// Returns `Ok(())` once the user confirms and the chunk file exists at
/// the expected path.
pub fn prompt_unpack_swap(source_dir: &Path, chunk_filename: &str) -> Result<()> {
    loop {
        prompt_usb_swap(&format!(
            "{} not found. Please insert the USB drive containing this chunk at {}.",
            chunk_filename,
            source_dir.display()
        ))?;

        if source_dir.join(chunk_filename).exists() {
            println!("  {} {} found.", "✓".green().bold(), chunk_filename);
            return Ok(());
        }

        println!(
            "  {} {} still not found. Try again.",
            "!".red().bold(),
            chunk_filename
        );
    }
}
