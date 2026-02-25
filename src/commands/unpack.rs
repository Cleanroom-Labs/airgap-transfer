/// Unpack command — reconstruct files from chunks (Phase 5, not yet implemented).
use std::path::Path;

use crate::error::Result;

/// Execute the unpack operation.
pub fn run(
    _source: &Path,
    _dest: &Path,
    _no_verify: bool,
    _keep_chunks: bool,
    _verbose: bool,
) -> Result<()> {
    eprintln!("unpack command is not yet implemented (planned for Phase 5)");
    std::process::exit(1);
}
