# Repository Guidelines

## Project Structure & Module Organization
Core Rust code lives in `src/`.
- `src/main.rs` is the CLI entrypoint.
- `src/lib.rs` exposes shared modules (`chunker`, `manifest`, `verifier`, `usb`, etc.).
- `src/commands/` contains subcommand implementations (`pack`, `unpack`, `list`).
Integration tests live in `tests/` (for end-to-end CLI flows), while unit tests are colocated in source files under `#[cfg(test)]`. Documentation sources are in `docs/source/`; shared spec assets are in the `docs/spec-docs/` submodule.

## Build, Test, and Development Commands
Use these from repo root:
- `cargo build` - build debug binaries.
- `cargo test` - run unit + integration tests.
- `cargo clippy -- -D warnings` - lint; warnings fail CI.
- `cargo fmt --check` - formatting check.
- `cargo run -- pack <source> <dest> --chunk-size 1GB` - run CLI locally.
- `make -C docs html` - build docs (`docs/_build/html`).
- `make -C docs test-e2e-only` - run docs Playwright tests.

## Coding Style & Naming Conventions
Rust style is enforced by `rustfmt` and `clippy`; keep code warning-free.
- Indentation: 4 spaces (Rust default).
- Naming: `snake_case` for functions/modules, `PascalCase` for types/structs/enums, `SCREAMING_SNAKE_CASE` for constants.
- Prefer small, focused modules and explicit error propagation (`Result`, project error types).
- In tests, keep the existing traceability pattern: `/// Spec: TC-...` above relevant test cases.

## Testing Guidelines
Add unit tests next to implementation and integration scenarios in `tests/*.rs`.
Name tests descriptively (for example, `pack_aborts_without_force`).
Before opening a PR, run:
1. `cargo test`
2. `cargo clippy -- -D warnings`
3. `cargo fmt --check`
For docs-impacting changes, also run `make -C docs html`.

## Commit & Pull Request Guidelines
Follow Conventional Commit style seen in history: `feat:`, `fix:`, `chore:`, with optional scopes like `chore(docs):`.
Keep commits small and single-purpose. PRs should include:
- What changed and why.
- Any security/behavioral impact.
- Linked issue (if available).
- Updated docs/tests for CLI or workflow changes.

## Security & Local Hooks
Install hooks once per clone: `git config core.hooksPath .githooks`.
Do not commit secrets, private keys, `.env` files, or machine-specific paths. Semgrep exclusions are centralized in `.semgrepignore`.
