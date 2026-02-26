# AirGap Transfer

A minimal command-line utility for safely transferring large files and datasets across air-gap boundaries using removable media.

## Features

- **Air-gap ready** — Designed for systems with no network access
- **Chunked transfers** — Split large datasets across multiple USB drives
- **Integrity verification** — SHA-256 checksums for all transfers (configurable algorithm)
- **Resume capability** — Continue interrupted transfers
- **Cross-platform** — macOS, Windows, and Linux
- **Lightweight** — Pure Rust, single binary

## How It Works

1. **Pack** — Split source files into chunks that fit on available USB drives
2. **Transfer** — Physically move USB drives across air-gap boundary
3. **Unpack** — Reconstruct original files on destination machine with verification

All operations maintain data integrity through cryptographic checksums.

## Quick Start

### Prerequisites

- Rust toolchain (for building from source)
- USB drives with sufficient combined capacity

### Installation

```bash
cargo build --release
```

### Usage

```bash
# On source machine — split files into USB-sized chunks:
airgap-transfer pack ~/large-dataset /media/usb-drive --chunk-size 16GB

# Physically transfer USB drive(s) across air-gap

# On destination machine — reconstruct with verification:
airgap-transfer unpack /media/usb-drive ~/restored-dataset

# Check chunk inventory and status:
airgap-transfer list /media/usb-drive
```

### Commands

| Command | Description |
|---------|-------------|
| `pack <source> <dest>` | Split files into chunks |
| `unpack <source> <dest>` | Reconstruct from chunks |
| `list <chunk-location>` | Show chunk inventory |

### Flags

| Flag | Description |
|------|-------------|
| `--chunk-size <SIZE>` | Manual chunk size (default: auto-detect USB capacity) |
| `--hash-algorithm <ALG>` | Hash algorithm (default: sha256) |
| `--dry-run` | Preview operations without writing |
| `--no-verify` | Skip checksum verification |
| `--verbose` | Detailed output |

## Air-Gapped Deployment

All dependencies can be vendored for offline builds:

```bash
cargo vendor
cargo build --release --offline
```

## Privacy

AirGap Transfer is **private by architecture**:

- Zero network code in the application
- No analytics, telemetry, or external API calls
- All data stays on local or removable media

## Documentation

Detailed specifications live in the [`docs/`](docs/) submodule:

| Document | Purpose |
|----------|---------|
| [Requirements (SRS)](docs/source/requirements/srs.rst) | Functional and non-functional requirements |
| [Design (SDD)](docs/source/design/sdd.rst) | Architecture and component design |
| [Test Plan](docs/source/testing/plan.rst) | Test cases and procedures |
| [Roadmap](docs/source/roadmap.md) | Implementation milestones |

## Development

```bash
cargo build                    # Debug build
cargo test                     # Run all tests
cargo clippy -- -D warnings    # Lint
cargo fmt --check              # Format check
```

### Supply-chain security

Five tools provide layered dependency assurance:

| Tool | Purpose | Install |
|------|---------|---------|
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | License compliance, advisory checks, source restrictions, duplicate dep detection | `cargo install cargo-deny` |
| [cargo-vet](https://github.com/mozilla/cargo-vet) | Verifies deps have been reviewed by trusted auditors (Mozilla, Google) | `cargo install cargo-vet` |
| [cargo-crev](https://github.com/crev-dev/cargo-crev) | Decentralized community code reviews with cryptographic signatures | `cargo install cargo-crev` |
| [cargo-auditable](https://github.com/rust-secure-code/cargo-auditable) | Embeds dependency metadata in compiled binaries for offline auditing | `cargo install cargo-auditable` |
| [cargo-audit](https://github.com/rustsec/rustsec) | Scans Cargo.lock or built binaries against RustSec advisories | `cargo install cargo-audit` |

```bash
cargo deny check                                  # License, advisory, source, and ban checks
cargo deny list                                    # Show all dependency licenses
cargo vet                                          # Verify all deps have trusted human audits
cargo crev crate verify                            # Community trust check
cargo auditable build --release                    # Release build with embedded dep manifest
cargo audit bin target/release/airgap-transfer     # Scan built binary for vulnerabilities
```

Configuration lives in `deny.toml` (cargo-deny) and `supply-chain/` (cargo-vet). CI enforces cargo-deny and cargo-vet on every push/PR (with weekly advisory scans), runs cargo-crev as an advisory check, and builds an auditable release binary.

## License

AGPL-3.0
