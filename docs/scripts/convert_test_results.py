#!/usr/bin/env python3
"""Convert ``cargo test --format json`` output to a sphinx-needs JSON file.

Usage::

    cargo +nightly test -- --format json -Z unstable-options 2>/dev/null \
      | python3 docs/scripts/convert_test_results.py \
          --mapping docs/test-mapping.json \
          --output docs/source/_data/test-results.json

The output file can be consumed by ``.. needimport::`` in the Sphinx build.
"""

import argparse
import json
import sys
from collections import defaultdict
from datetime import datetime, timezone


def load_mapping(path: str) -> dict[str, list[str]]:
    """Load test-mapping.json → {rust_test: [spec_id, ...]}."""
    with open(path) as f:
        data = json.load(f)
    mapping: dict[str, list[str]] = defaultdict(list)
    for entry in data["mappings"]:
        mapping[entry["rust_test"]].append(entry["spec_id"])
    return dict(mapping)


def parse_cargo_json(stream) -> dict[str, str]:
    """Parse cargo test JSON lines → {test_name: 'passed'|'failed'|'ignored'}.

    cargo test --format json emits one JSON object per line.  We look for
    events with ``type == "test"`` and ``event`` in {ok, failed, ignored}.
    """
    results: dict[str, str] = {}
    for line in stream:
        line = line.strip()
        if not line:
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("type") == "test" and "event" in obj:
            event = obj["event"]
            name = obj.get("name", "")
            if event == "ok":
                results[name] = "passed"
            elif event == "failed":
                results[name] = "failed"
            elif event == "ignored":
                results[name] = "ignored"
    return results


def build_needs_json(
    test_results: dict[str, str],
    mapping: dict[str, list[str]],
) -> dict:
    """Build a sphinx-needs JSON structure from test results and mapping.

    Each spec test case gets a need with status reflecting the Rust test
    outcome.  If multiple Rust tests map to the same spec ID, the worst
    status wins (failed > ignored > passed).
    """
    STATUS_PRIORITY = {"failed": 0, "ignored": 1, "passed": 2}

    # Aggregate per spec_id
    spec_status: dict[str, str] = {}
    mapped_rust: set[str] = set()
    for rust_test, spec_ids in mapping.items():
        if rust_test in test_results:
            mapped_rust.add(rust_test)
            for spec_id in spec_ids:
                new = test_results[rust_test]
                if spec_id not in spec_status:
                    spec_status[spec_id] = new
                else:
                    current = spec_status[spec_id]
                    if STATUS_PRIORITY.get(new, 3) < STATUS_PRIORITY.get(current, 3):
                        spec_status[spec_id] = new

    # Report unmapped tests
    unmapped = set(test_results.keys()) - mapped_rust
    if unmapped:
        print(f"Warning: {len(unmapped)} unmapped Rust test(s):", file=sys.stderr)
        for t in sorted(unmapped):
            print(f"  - {t}", file=sys.stderr)

    # Build needs list
    now = datetime.now(timezone.utc).isoformat()
    needs: dict[str, dict] = {}
    for spec_id, status in sorted(spec_status.items()):
        need_id = f"RESULT-{spec_id}"
        needs[need_id] = {
            "id": need_id,
            "type": "test",
            "title": f"Test Result: {spec_id}",
            "status": status,
            "tags": ["ci-result"],
            "description": f"CI test result for {spec_id} (auto-generated {now})",
            "links": [spec_id],
        }

    return {
        "created": now,
        "current_version": "current",
        "versions": {
            "current": {
                "created": now,
                "needs": needs,
            }
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--mapping",
        required=True,
        help="Path to test-mapping.json",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output path for the needs.json file",
    )
    parser.add_argument(
        "--input",
        default="-",
        help="Input file (default: stdin, for piping cargo test output)",
    )
    args = parser.parse_args()

    mapping = load_mapping(args.mapping)

    if args.input == "-":
        test_results = parse_cargo_json(sys.stdin)
    else:
        with open(args.input) as f:
            test_results = parse_cargo_json(f)

    if not test_results:
        print("Warning: no test results found in input", file=sys.stderr)

    needs_json = build_needs_json(test_results, mapping)

    with open(args.output, "w") as f:
        json.dump(needs_json, f, indent=2)

    total = len(needs_json["versions"]["current"]["needs"])
    passed = sum(
        1
        for n in needs_json["versions"]["current"]["needs"].values()
        if n["status"] == "passed"
    )
    failed = sum(
        1
        for n in needs_json["versions"]["current"]["needs"].values()
        if n["status"] == "failed"
    )
    ignored = sum(
        1
        for n in needs_json["versions"]["current"]["needs"].values()
        if n["status"] == "ignored"
    )
    print(f"Wrote {total} test results ({passed} passed, {failed} failed, {ignored} ignored)")


if __name__ == "__main__":
    main()
