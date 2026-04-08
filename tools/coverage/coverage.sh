#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <test_name>"
    exit 1
fi

TEST_NAME="$1"

COV_DIR="target/llvm-cov-target"
TMP_DIR="$COV_DIR/tmp-json"
FINAL_FILE="$COV_DIR/final_coverage.json"

rm -fr "$COV_DIR"
mkdir -p "$TMP_DIR"

export RUSTFLAGS="-C instrument-coverage"

TEST_NAME_CLEAN="${TEST_NAME%:}"
echo "Running test: $TEST_NAME_CLEAN"

export LLVM_PROFILE_FILE="$COV_DIR/${TEST_NAME_CLEAN//::/_}/%p-%m.profraw"
rm -f "$COV_DIR/${TEST_NAME_CLEAN//::/_}" "$COV_DIR/"*.profraw || true
mkdir -p "$COV_DIR/${TEST_NAME_CLEAN//::/_}"
echo "$LLVM_PROFILE_FILE"

TEST_LOG="$TMP_DIR/${TEST_NAME_CLEAN//::/_}.log"

cargo llvm-cov test "$TEST_NAME_CLEAN" -- --nocapture > "$TEST_LOG" 2>&1

RAW_JSON_LINES=$(grep "mantra: req-id=" "$TEST_LOG" | sed -nE "s/.*req-id=\`([^\`]+)\`; file='([^']+)'; line='([0-9]+)';.*/{\"req_id\":\"\1\", \"file\":\"\2\", \"line\":\3}/p" || true)

if [ -z "$RAW_JSON_LINES" ]; then
    TRACES_JSON="[]"
else
    TRACES_JSON=$(echo "$RAW_JSON_LINES" | jq -s '.')
fi

echo "TRACES: $TRACES_JSON"

TEST_JSON="$TMP_DIR/${TEST_NAME_CLEAN//::/_}.json"
cargo llvm-cov report --json > "$TEST_JSON"

FRAGMENT="$TMP_DIR/${TEST_NAME_CLEAN//::/_}_coverage_fragment.json"
jq --arg test_name "$TEST_NAME_CLEAN" \
    --argjson traces "$TRACES_JSON" \
    --arg pwd "$PWD/" \
    -f transform_to_coverage_schema.jq "$TEST_JSON" > "$FRAGMENT"

echo "Written coverage fragment for $TEST_NAME_CLEAN"

jq -s -f merge_fragments.jq $TMP_DIR/*_coverage_fragment.json > "$FINAL_FILE"
echo "Merged coverage file created at $FINAL_FILE"

echo "Collecting all new information"
mantra collect mantra.toml

echo "Generating new report"
mantra report --formats=html output.html