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
cargo deny check               # License, advisory, source, and duplicate dep audit (requires: cargo install cargo-deny)
cargo vet                      # Verify all deps have trusted audits (requires: cargo install cargo-vet)
cargo crev crate verify        # Community trust check (requires: cargo install cargo-crev)
cargo auditable build --release # Release build with embedded dep metadata (requires: cargo install cargo-auditable)
cargo audit bin target/release/airgap-transfer  # Scan built binary for advisories (requires: cargo install cargo-audit)
cargo geiger                   # Audit unsafe code in dep tree
trivy fs .                     # Scan for NVD/GHSA/OSV vulnerabilities (requires trivy installed)
```

### Supply-chain security

Seven tools provide layered dependency assurance:

- **cargo-deny** (`deny.toml`) — Build-time gate: license allowlisting (AGPL-compatible only), RustSec advisory checks, source restrictions (crates.io only), duplicate version detection. CI runs on every push/PR and weekly for new advisories.
- **cargo-vet** (`supply-chain/`) — Verifies dependencies have been reviewed by trusted auditors (Mozilla, Google). Centralized trust model. Blocking in CI. Uses exemptions for unaudited deps, which are gradually reduced as imported audit coverage grows.
- **cargo-crev** — Decentralized community code reviews with cryptographic signatures. Web of trust model. Advisory in CI (non-blocking) due to sparse coverage.
- **cargo-auditable** — Embeds a compressed dependency manifest (~4KB) into the compiled binary. Enables offline auditing of deployed binaries without source access.
- **cargo-audit** — Scans Cargo.lock or compiled binaries (`cargo audit bin`) against RustSec advisories. Also offers `cargo audit fix` for auto-resolving advisories.
- **cargo-geiger** — Counts unsafe code expressions in the full dependency tree. CI enforces a fail-on-increase policy on PRs: if a PR adds unsafe expressions compared to the main branch baseline, the geiger check fails. Run locally with `cargo geiger` for situational awareness before pushing.
- **trivy** — Scans the auditable binary and project files against NVD (NIST), GitHub Security Advisories (GHSA), and OSV (which includes RustSec). Complements `cargo-audit`: cargo-audit covers RustSec advisories; trivy covers NVD and GHSA advisories that RustSec may not include. Both are required for full vulnerability coverage.

**CI/CD workflows:** Three scheduled workflows handle different cadences — `ci.yml` (push/PR: all quality and supply-chain checks), `security.yml` (weekly Monday: advisory scans + Trivy + geiger baseline), `maintenance.yml` (monthly first Wednesday: build freshness + cargo-outdated + cargo-duplicates + all security jobs). A fourth workflow, `dep-summary.yml`, posts Claude-generated changelog summaries on Dependabot PRs. See `CONTRIBUTING.md` for the full CI/CD guide.

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

## Debugging

When encountering bugs or test failures, investigate the root cause before attempting fixes. Do not cycle through surface-level patches — read the relevant code, trace the execution path, and identify the actual cause before proposing changes.

## Documentation

Full specifications are in the `docs/spec-docs/` submodule (Sphinx project):

- `docs/spec-docs/source/requirements/srs.rst` — Software Requirements Specification
- `docs/spec-docs/source/design/sdd.rst` — Software Design Document
- `docs/spec-docs/source/testing/plan.rst` — Test Plan
- `docs/spec-docs/source/roadmap.md` — Project Roadmap

### Building docs locally

```bash
git submodule update --init --recursive
pip install -r docs/requirements.txt
make -C docs html                     # Build unified site → docs/_build/html/
make -C docs full                     # Build with test results (requires Rust nightly)
```
