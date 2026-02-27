/// Shared helpers for integration tests.
use assert_cmd::Command;

/// Build a `Command` for the `airgap-transfer` binary.
pub fn cmd() -> Command {
    Command::cargo_bin("airgap-transfer").unwrap()
}
