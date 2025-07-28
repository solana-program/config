#!/usr/bin/env bash

# Mollusk regression testing of this Core BPF Config program against the
# version running on mainnet-beta.

set -euo pipefail

WORKDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

PROGRAM_ID="Config1111111111111111111111111111111111111"
PROGRAM_BINARY_PATH="$WORKDIR/target/deploy/solana_config_program.so"
BASE_BINARY_PATH="$WORKDIR/program/fuzz/program-mb-3-17-2025.so"
MOLLUSK_FIXTURES_PATH="$WORKDIR/program/fuzz/blob"

# Test this program against the cloned program for regression with Mollusk.
OUTPUT=$(mollusk run-test \
    --proto mollusk --ignore-compute-units \
    "$BASE_BINARY_PATH" "$PROGRAM_BINARY_PATH" \
    "$MOLLUSK_FIXTURES_PATH" "$PROGRAM_ID")

# The last line of output should exactly match the following:
# [DONE][TEST RESULT]: 0 failures
if ! grep -q "\[DONE\]\[TEST RESULT\]: 0 failures" <<< "$OUTPUT"; then
    echo "Error: mismatches detected."
    exit 1
fi

echo "$OUTPUT"
echo "Regression test passed."