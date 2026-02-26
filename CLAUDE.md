# CLAUDE.md

## Project Overview

AirGap Transfer is a Rust CLI tool for transferring large files across air-gap boundaries using removable media (USB drives). It splits data into chunks, tracks state via a JSON manifest, and verifies integrity with pluggable cryptographic checksums.

## Architecture

Pure Rust CLI application with streaming architecture (memory < 100MB). Flat file structure per project principles:

```
src/
├── main.rs              # Entry point, CLI setup (clap)
├── commands/
│   ├── pack.rs          # Pack operation — split files into chunks
│   ├── unpack.rs        # Unpack operation — reconstruct from chunks
│   └── list.rs          # List operation — show chunk inventory
├── chunker.rs           # Streaming chunk creation/reconstruction (tar format)
├── verifier.rs          # Pluggable hash verification (HashAlgorithm trait)
├── manifest.rs          # JSON manifest — metadata and state persistence
└── usb.rs               # USB detection and capacity checks (platform-specific)
```

### Key Design Decisions

- **Streaming**: Data streams directly to/from USB — no intermediate temp files
- **Trait-based hashing**: `HashAlgorithm` trait for pluggable backends. SHA-256 default, extensible to future algorithms
- **JSON manifest**: `airgap-transfer-manifest.json` tracks chunks, checksums, and operation state. Enables resume
- **Tar chunks**: Chunks are `chunk_XXX.tar` files (zero-padded 3-digit index)
- **No network code**: Zero network crates in dependency tree. Privacy by architecture
- **Verification on by default**: `--no-verify` disables; checksums are never silently skipped

## Dependencies

Target ≤10 direct crates: `clap`, `serde`, `serde_json`, `sha2`, `tar`, `thiserror`, `colored`, `indicatif`.

Vendored dependencies supported via `cargo vendor` for air-gap builds.

## Build & Test

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo build --release --offline # Air-gap build (after cargo vendor)
cargo test                     # Run all tests
cargo clippy -- -D warnings    # Lint (must pass with zero warnings)
cargo fmt --check              # Format check
cargo deny check               # License + advisory audit (requires: cargo install cargo-deny)
```

`deny.toml` configures license allowlisting (AGPL-compatible only), security advisory checks, and source restrictions (crates.io only). CI runs these checks on every push/PR and weekly for new advisories.

## Code Quality Standards

- Zero clippy warnings (`cargo clippy -- -D warnings`)
- All code formatted with `rustfmt`
- 80%+ test coverage target
- All public APIs must have rustdoc documentation
- AGPL-3.0 license — all dependency licenses must be compatible (enforce with `cargo-deny`)

## Manifest Schema

```json
{
  "version": "1.0",
  "operation": "pack",
  "source_path": "/path/to/source",
  "total_size_bytes": 10737418240,
  "chunk_size_bytes": 1073741824,
  "hash_algorithm": "sha256",
  "chunk_count": 10,
  "chunks": [
    {
      "index": 0,
      "filename": "chunk_000.tar",
      "size_bytes": 1073741824,
      "checksum": "sha256:abc123...",
      "status": "completed"
    }
  ],
  "created_utc": "2026-01-04T12:00:00Z",
  "last_updated_utc": "2026-01-04T12:15:00Z"
}
```

## CLI Interface

```
airgap-transfer pack <source> <dest> [--chunk-size SIZE] [--hash-algorithm ALG] [--dry-run] [--no-verify] [--verbose]
airgap-transfer unpack <source> <dest> [--no-verify] [--keep-chunks] [--verbose]
airgap-transfer list <chunk-location>
```

## Platform-Specific Notes

| Platform | USB Detection | Filesystem Sync |
|----------|---------------|-----------------|
| macOS | `/Volumes/*` | `sync` syscall |
| Linux | `/media/$USER/*` or `/mnt/*` | `sync` syscall |
| Windows | DriveInfo API via WinAPI | `FlushFileBuffers` API |

## Documentation

Full specifications are in the `docs/` submodule (Sphinx project):

- `docs/source/requirements/srs.rst` — Software Requirements Specification
- `docs/source/design/sdd.rst` — Software Design Document
- `docs/source/testing/plan.rst` — Test Plan
- `docs/source/roadmap.md` — Project Roadmap
