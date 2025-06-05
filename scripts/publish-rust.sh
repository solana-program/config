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

ABS_MANIFEST_PATH=$(realpath $library_path/Cargo.toml)

# Function for obtaining manifest fields from the Cargo.toml.
get_manifest_field() {
    local field="$1"
    cargo metadata --no-deps --format-version 1 \
        | jq --arg path "$ABS_MANIFEST_PATH" -r ".packages[] | select(.manifest_path == \$path) | .${field}"
}

cd "$library_path"

# Extract crate name
crate_name=$(get_manifest_field "name")

# Run cargo-release
if [[ "$dry_run" != "true" ]]; then
    cargo release "$level" --tag-name "${crate_name}@v{{version}}" --no-confirm --execute --dependent-version fix
else
    cargo release "$level"
    exit 0
fi

# CI output
if [[ -n "${CI:-}" ]]; then
    new_version=$(get_manifest_field "version")
    echo "new_version=${new_version}" >> "$GITHUB_OUTPUT"
fi
