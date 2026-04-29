#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
GENERATOR_DIR="$ROOT_DIR/dataset/generator"
OUT_DIR="$ROOT_DIR/dataset/generated"

echo "=== AgentGauntlet Dataset Generator ==="
echo "Output: $OUT_DIR"
echo ""

mkdir -p "$OUT_DIR"

cd "$GENERATOR_DIR"

if command -v uv &>/dev/null; then
    uv run python generate_scenarios.py --count-per-category 30 --out "$OUT_DIR"
elif command -v python3 &>/dev/null; then
    pip install pyyaml jinja2 -q
    python3 generate_scenarios.py --count-per-category 30 --out "$OUT_DIR"
else
    echo "ERROR: Neither 'uv' nor 'python3' found. Install Python 3.11+ or uv."
    exit 1
fi

echo ""
echo "Dataset generation complete."
echo "Files: $(find "$OUT_DIR" -name '*.yaml' | wc -l) scenarios generated"
