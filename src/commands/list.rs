/// List command — display chunk inventory (Phase 5, not yet implemented).
use std::path::Path;

use crate::error::Result;

/// Execute the list operation.
pub fn run(_chunk_location: &Path) -> Result<()> {
    eprintln!("list command is not yet implemented (planned for Phase 5)");
    std::process::exit(1);
}
