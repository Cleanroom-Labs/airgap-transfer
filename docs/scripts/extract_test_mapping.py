#!/usr/bin/env python3
"""Extract ``/// Spec: TC-XXX`` annotations from Rust source files.

Walks ``src/`` and ``tests/`` directories, finds test functions annotated
with ``/// Spec: TC-XXX`` doc comments, resolves the full cargo-test name,
and writes the result as ``test-mapping.json`` in the format consumed by
``convert_test_results.py``.

Usage::

    python3 docs/scripts/extract_test_mapping.py \
        --src-dir src --test-dir tests --output docs/test-mapping.json
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

# Matches ``/// Spec: TC-XXX`` with optional comma-separated extra IDs.
SPEC_RE = re.compile(r"^\s*///\s*Spec:\s*(TC-[\w-]+(?:\s*,\s*TC-[\w-]+)*)")

# Matches ``fn function_name(`` after ``#[test]``.
FN_RE = re.compile(r"^\s*fn\s+(\w+)\s*\(")

# Matches ``#[test]`` attribute.
TEST_RE = re.compile(r"^\s*#\[test\]")


def module_path_for_src(file_path: Path, src_dir: Path) -> str:
    """Derive the cargo-test module prefix for a unit test in *src/*.

    Examples::

        src/chunker.rs        -> chunker::tests::
        src/commands/pack.rs   -> commands::pack::tests::
        src/main.rs            -> tests::
    """
    relative = file_path.relative_to(src_dir)
    stem = str(relative.with_suffix("")).replace("/", "::")

    # main.rs tests appear as ``tests::fn_name`` in cargo output.
    if stem == "main":
        return "tests::"

    return f"{stem}::tests::"


def extract_from_file(
    file_path: Path,
    prefix: str,
) -> tuple[list[dict], list[str]]:
    """Parse a single ``.rs`` file, returning (mappings, warnings)."""

    mappings: list[dict] = []
    warnings: list[str] = []

    lines = file_path.read_text().splitlines()

    spec_ids: list[str] | None = None
    expecting_test = False
    expecting_fn = False

    for lineno, line in enumerate(lines, start=1):
        # Look for ``/// Spec: TC-XXX`` lines.
        m = SPEC_RE.match(line)
        if m:
            ids_str = m.group(1)
            spec_ids = [s.strip() for s in ids_str.split(",")]
            continue

        # After a Spec comment, the next meaningful line should be #[test].
        if spec_ids is not None and TEST_RE.match(line):
            expecting_fn = True
            continue

        # After #[test], the next meaningful line should be fn name().
        if expecting_fn:
            m = FN_RE.match(line)
            if m:
                fn_name = m.group(1)
                for sid in spec_ids:
                    mappings.append(
                        {"rust_test": f"{prefix}{fn_name}", "spec_id": sid}
                    )
                spec_ids = None
                expecting_fn = False
                continue

        # Detect #[test] without a Spec annotation.
        if TEST_RE.match(line):
            expecting_test = True
            continue

        if expecting_test:
            m = FN_RE.match(line)
            if m:
                fn_name = m.group(1)
                warnings.append(
                    f"{file_path}:{lineno}: test `{fn_name}` has no /// Spec annotation"
                )
                expecting_test = False
                continue

        # Reset state on non-matching lines (skip blank lines and other
        # doc comments between Spec and #[test]).
        if spec_ids is not None and not line.strip().startswith("///") and line.strip():
            # Non-doc-comment, non-blank line before #[test] — reset.
            spec_ids = None

        if expecting_test and not line.strip().startswith("///") and line.strip():
            expecting_test = False

    return mappings, warnings


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--src-dir",
        type=Path,
        default=Path("src"),
        help="Path to the src/ directory (default: src)",
    )
    parser.add_argument(
        "--test-dir",
        type=Path,
        default=Path("tests"),
        help="Path to the tests/ directory (default: tests)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("docs/test-mapping.json"),
        help="Output JSON path (default: docs/test-mapping.json)",
    )
    args = parser.parse_args()

    all_mappings: list[dict] = []
    all_warnings: list[str] = []

    # Unit tests in src/
    for rs_file in sorted(args.src_dir.rglob("*.rs")):
        prefix = module_path_for_src(rs_file, args.src_dir)
        mappings, warnings = extract_from_file(rs_file, prefix)
        all_mappings.extend(mappings)
        all_warnings.extend(warnings)

    # Integration tests in tests/
    for rs_file in sorted(args.test_dir.rglob("*.rs")):
        # Skip helper modules (e.g. tests/common/mod.rs)
        if rs_file.parent.name != args.test_dir.name:
            continue
        # Integration tests use bare function names in cargo output.
        mappings, warnings = extract_from_file(rs_file, "")
        all_mappings.extend(mappings)
        all_warnings.extend(warnings)

    # Print warnings to stderr
    for w in all_warnings:
        print(f"WARNING: {w}", file=sys.stderr)

    # Write output
    output = {
        "$comment": "Auto-generated from /// Spec: annotations. Do not edit manually.",
        "mappings": all_mappings,
    }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n")
    print(
        f"Wrote {len(all_mappings)} mappings to {args.output}", file=sys.stderr
    )

    if all_warnings:
        print(
            f"{len(all_warnings)} unannotated test(s) found", file=sys.stderr
        )


if __name__ == "__main__":
    main()
