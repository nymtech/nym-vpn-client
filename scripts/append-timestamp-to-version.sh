#!/usr/bin/env bash
#
# Usage:
#   set_version.sh <manifest-path> [<current-version>]
#
#   <manifest-path>    Path to the Cargo.toml (required).
#   <current-version>  If not provided, the script will call cargo-get to retrieve the version.

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <manifest-path> [<current-version>]"
    exit 1
fi

MANIFEST_PATH="$1"
CURRENT_VERSION="${2:-}"

# If current version is empty, retrieve it using cargo-get
if [[ -z "$CURRENT_VERSION" ]]; then
    echo "No version argument supplied. Using cargo-get to retrieve version from $MANIFEST_PATH ..."
    CURRENT_VERSION="$(cargo get workspace.package.version --entry "$MANIFEST_PATH")"
fi

echo "Current version is: $CURRENT_VERSION"

# Check if the version ends with "-dev". If so, append a timestamp
if [[ "$CURRENT_VERSION" == *"-dev" ]]; then
    TIMESTAMP="$(date +'%Y%m%d%H%M%S')"
    NEW_VERSION="${CURRENT_VERSION}.${TIMESTAMP}"
else
    NEW_VERSION="$CURRENT_VERSION"
fi

echo "Setting version to: $NEW_VERSION"

# Use cargo-edit to update the Cargo.toml
cargo set-version --manifest-path "$MANIFEST_PATH" "$NEW_VERSION"

# Export the final version to GitHub Actions (if used within a workflow)
if [[ -n "${GITHUB_OUTPUT-}" ]]; then
    echo "workspace_version=$NEW_VERSION" >> "$GITHUB_OUTPUT"
fi

