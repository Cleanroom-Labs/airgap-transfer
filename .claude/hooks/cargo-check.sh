#!/bin/bash
# Post-edit hook: run cargo check after Rust source file edits
set -euo pipefail

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('file_path',''))" 2>/dev/null || echo "")

# Only run for .rs files under src/
case "$FILE_PATH" in
  */src/*.rs)
    cargo check 2>&1 | tail -5
    ;;
esac
