# Contributing to AirGap Transfer

## 1. Getting Started

**Prerequisites:** Rust stable toolchain (`rustup` recommended).

```bash
git clone https://github.com/Cleanroom-Labs/airgap-transfer.git
cd airgap-transfer
cargo build
cargo test
```

---

## 2. Before You Push — Local Checks

Run these commands locally before pushing. CI enforces all of them and will fail if they do not pass.

```bash
cargo clippy -- -D warnings    # Zero warnings required
cargo fmt --check               # Format check
cargo deny check                # License, advisory, source checks
cargo vet                       # Dependency audit verification
cargo geiger                    # Informational: understand your unsafe code footprint
trivy fs .                      # Informational: scan for NVD/GHSA vulnerabilities locally
```

`cargo geiger` is informational when run locally, but CI enforces that PRs do not increase the unsafe expression count compared to the main branch baseline. See [Handling CI Failures](#5-handling-ci-failures) and [Adding Justified Unsafe Code](#7-adding-justified-unsafe-code) for details.

---

## 3. Building Documentation Locally

The project documentation (specifications, traceability dashboard, implementation mapping) is built with Sphinx and deployed to GitHub Pages by CI. You can also build it locally.

**Prerequisites:** Python 3 and pip. Rust nightly is only needed if you want test results in the dashboard.

```bash
git submodule update --init --recursive   # Initialize docs submodules
pip install -r docs/requirements.txt      # Install Sphinx + extensions
make -C docs html                         # Build → docs/_build/html/
open docs/_build/html/index.html          # View locally
```

To include live test results in the dashboard:

```bash
make -C docs full
```

Both `docs/test-mapping.json` and `docs/source/_data/test-results.json` are gitignored — CI generates them fresh on each deploy. The test mapping is extracted from `/// Spec: TC-XXX` annotations on test functions. The docs build succeeds without either file; the dashboard will show specification data but no test pass/fail results.

---

## 4. CI/CD Overview

| Workflow | Trigger | Jobs | Purpose |
|---|---|---|---|
| `ci.yml` | push to main, pull request to main | check (3-OS), lint, docs, deny, vet, crev, auditable, geiger (PRs only) | Core quality gate — every change is validated; geiger runs only on pull requests |
| `security.yml` | weekly (Monday 6am CST) | deny, vet, crev, auditable, trivy, geiger-baseline | Advisory scan — catches newly-disclosed vulnerabilities between dependency updates |
| `maintenance.yml` | monthly (first Wednesday 6am CST) | check (3-OS), lint, outdated, duplicates, deny, vet, crev, auditable, trivy, geiger-baseline | Freshness check — verifies build health and surfaces dependency staleness |
| `dep-summary.yml` | Dependabot PRs | summarize | AI-assisted changelog analysis — posts a project-relevant summary on each Dependabot PR |
| `static.yml` | push to main | deploy | Builds Sphinx docs and deploys to GitHub Pages |

The CI pipeline takes a defense-in-depth approach to supply-chain security. Each tool covers a different trust model and vulnerability database — no single tool is sufficient alone. `cargo-deny` enforces license and advisory policy at build time; `cargo-vet` ensures dependencies have been reviewed by trusted auditors; `cargo-crev` adds decentralized community trust signals; `cargo-auditable` and `cargo-audit` cover the compiled binary; and `trivy` extends advisory coverage beyond RustSec to NVD and GitHub Security Advisories. Together they provide overlapping assurance that no one gap leaves the project exposed.

---

## 5. Vulnerability Coverage Matrix

| Tool | Database / Source | What It Catches |
|---|---|---|
| `cargo-deny` | RustSec advisory database | Advisories, license violations, banned crates, non-crates.io sources |
| `cargo-vet` | Mozilla + Google audit imports | Dependencies that haven't been reviewed by trusted auditors |
| `cargo-crev` | Decentralized community reviews | Community trust signals (advisory — not blocking) |
| `cargo-audit` | RustSec | Vulnerabilities in the compiled binary's embedded manifest |
| `trivy` | NVD (NIST), GitHub Security Advisories (GHSA), OSV | Broad CVE coverage — complements RustSec for NVD/GHSA-specific advisories |
| `cargo-geiger` | Source analysis | Unsafe Rust expressions across the full dependency tree |

---

## 6. Handling CI Failures

**cargo-deny failure**

A new advisory was published against one of the project's dependencies. Run `cargo deny check` locally to identify which advisory triggered the failure. Options:

- Update the affected dependency to a patched version.
- Add a time-limited denial exception in `deny.toml` with a rationale comment if no fix is yet available.

**cargo-vet failure**

A dependency was added or updated and lacks trusted auditor coverage. Run `cargo vet` locally — it will identify exactly which crate needs attention. Options:

- Add an exemption in `supply-chain/config.toml` with a justification comment.
- Wait for Mozilla/Google audits to cover it (imported automatically on the next `cargo vet` run).

**geiger failure (PR blocked)**

The PR adds unsafe Rust code, increasing the expression count above the main branch baseline. Options:

- If the unsafe code is justified (e.g., necessary FFI): merge the unsafe addition in a separate commit directly to main first, with a clear commit message explaining why it is safe. The new baseline is then established and subsequent PRs will pass. See [Adding Justified Unsafe Code](#8-adding-justified-unsafe-code).
- If the unsafe code is not justified: remove it and use safe alternatives.

**trivy MEDIUM/HIGH/CRITICAL (filesystem) or HIGH/CRITICAL (binary)**

The filesystem scan blocks on MEDIUM severity and above, while the binary scan blocks on HIGH severity and above. A vulnerability was found in a dependency. Run `trivy fs .` locally to reproduce. Options:

- Update the affected dependency to a patched version.
- Add a Trivy vulnerability exception in a `.trivyignore` file with a rationale comment and a tracked issue if no fix is yet available.

**crev failure**

Advisory only — this job has `continue-on-error: true` and never blocks merges. No action is required for the PR to land, though the signal is worth reviewing.

---

## 7. Dependabot PRs

When Dependabot opens a PR for a dependency update, the `dep-summary.yml` workflow runs and posts a Claude-generated summary comment on the PR. The summary covers:

- Breaking API changes relevant to this project
- Security fixes
- Unsafe code changes in the dependency
- New features that might be useful
- Whether human review is needed before merging

If the summary contains **HUMAN REVIEW REQUIRED**, a GitHub issue is also created and a `needs-human-review` label is added to the PR. Do not merge these automatically — review the linked issue and the Claude analysis first.

For routine patch or minor updates with a clean CI run and no human-review flag: merge at your discretion.

---

## 8. Adding Justified Unsafe Code

CI enforces that PRs do not increase the unsafe expression count versus the main branch baseline. Unsafe code cannot be introduced inside a regular PR — the geiger check will fail.

To add justified unsafe code:

1. Make the unsafe addition in a standalone commit directly to main (or via a focused PR that only adds the unsafe block and nothing else).
2. Write a clear commit message explaining: what the unsafe code does, why it cannot be done with safe Rust, and why it is safe in this specific context.
3. After that commit lands on main it becomes the new geiger baseline. Subsequent PRs that do not add further unsafe code will pass the check.

This policy ensures all unsafe code introductions are deliberate, isolated, and documented.

---

## 9. Maintainer Setup

The following one-time setup is required when deploying this project to a new GitHub
repository for the `dep-summary.yml` workflow to function fully.

### CLAUDE_API_KEY secret

The dependency update summary workflow calls the Anthropic Claude API directly. This
requires an API key from [console.anthropic.com](https://console.anthropic.com) — a
Claude Max or Pro subscription does not include API access; they are separate products
with separate billing.

**Recommended:** Create the secret at the **organization level** so all repos inherit
it without per-repo configuration:

1. Get a key from [console.anthropic.com](https://console.anthropic.com) → API Keys → Create Key
2. GitHub **Organization** → Settings → Secrets and variables → Actions → **New organization secret**
3. Name: `CLAUDE_API_KEY`, Repository access: **All repositories**

Alternatively, add it per-repo: GitHub repo → Settings → Secrets and variables → Actions → New repository secret.

Cost: `claude-haiku-4-5-20251001` at ~$0.001–0.003 per Dependabot PR is negligible.
Without this secret the workflow still completes, but summaries show "Analysis unavailable."

### `needs-human-review` label

When Claude flags a dependency update as requiring human review, the workflow adds a
`needs-human-review` label to the PR and creates a tracking issue. Create the label:

GitHub repo → **Issues** → **Labels** → **New label** → Name: `needs-human-review`

Without this label, the flag still appears in the PR comment, but label assignment and
issue creation are silently skipped.
