#!/usr/bin/env bash
set -euo pipefail

# This script updates all git-based dependencies in the workspace whose package names start with "nym-".
# It uses `cargo update` to refresh the Cargo.lock file.
#
# Usage:
#   ./update_nym_deps.sh [<commit-hash>]
#
# If a commit hash is provided, all matching dependencies will be pinned to
# that specific commit.
#
# If no commit hash is provided, all matching dependencies will be updated to
# their latest commits on their respective branches.
#
# Examples:
#   ./update_nym_deps.sh          # Update all nym- git dependencies to the latest
#   ./update_nym_deps.sh abc1234  # Update all nym- git dependencies to commit abc1234
#

NEW_COMMIT="${1:-}"
ROOT_MANIFEST_PATH="nym-vpn-core/Cargo.toml"

# Extract all nym- packages that are git dependencies
PACKAGES=$(cargo metadata --manifest-path "$ROOT_MANIFEST_PATH" --format-version=1 \
    | jq -r '
        .packages[]
        | select((.name | startswith("nym-")) and ((.source // "") | contains("git+")))
        | .name
    ')

# Start building a single cargo update command
CMD=("cargo" "update" "--manifest-path" "$ROOT_MANIFEST_PATH")

if [ -n "$NEW_COMMIT" ]; then
    CMD+=("--precise" "$NEW_COMMIT")
fi

for pkg in $PACKAGES; do
    CMD+=("-p" "$pkg")
done

# Now execute the single cargo update command
echo "Running: ${CMD[*]}"
"${CMD[@]}"
