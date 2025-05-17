#!/usr/bin/env bash

set -euo pipefail

library_path="${1:-}"
level="${2:-}"
dry_run="${DRY_RUN:-false}"

if [[ -z "$library_path" || -z "$level" ]]; then
    echo "Usage: $0 <library_path> <version-level>"
    echo "Example: $0 clients/rust patch"
    exit 1
fi

cd "$library_path"

# Extract crate name using cargo metadata
metadata=$(cargo metadata --no-deps --format-version 1)
crate_name=$(echo "$metadata" | jq -r '.packages[0].name')

# Run cargo-release
if [[ "$dry_run" != "true" ]]; then
    cargo release "$level" --tag-name "${crate_name}@v{{version}}" --no-confirm --execute --dependent-version fix
else
    cargo release "$level"
    exit 0
fi

# CI output
if [[ -n "${CI:-}" ]]; then
    metadata=$(cargo metadata --no-deps --format-version 1)
    new_version=$(echo "$metadata" | jq -r '.packages[0].version')
    echo "new_version=${new_version}" >> "$GITHUB_OUTPUT"
fi
