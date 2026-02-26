/// Unpack command — reconstruct files from chunks (Phase 5, not yet implemented).
use crate::UnpackArgs;
use crate::error::Result;

/// Execute the unpack operation.
pub fn run(_args: &UnpackArgs) -> Result<()> {
    eprintln!("unpack command is not yet implemented (planned for Phase 5)");
    std::process::exit(1);
}
