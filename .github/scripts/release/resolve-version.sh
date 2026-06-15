#!/usr/bin/env bash
# Resolve the release version + unified tag.
# Inputs (env): SHIP (true|false), CARGO_ENTRY (default nym-vpn-core/Cargo.toml)
# Outputs (GITHUB_OUTPUT): core_version, tag
set -euo pipefail

SHIP="${SHIP:-false}"
CARGO_ENTRY="${CARGO_ENTRY:-nym-vpn-core/Cargo.toml}"

CORE_VERSION="$(cargo-get workspace.package.version --entry "$CARGO_ENTRY")"
echo "::notice:: core version: ${CORE_VERSION}"

if [ "$SHIP" = "true" ]; then
  if [[ "$CORE_VERSION" == *beta* ]]; then
    echo "::error:: refusing to ship a beta version: ${CORE_VERSION}"
    exit 1
  fi
  TAG="nym-vpn-v${CORE_VERSION}"
else
  # Unique-per-run nightly tag. Native immutable releases forbid REUSING a tag
  # once it was published, so each nightly gets a fresh timestamp; the prior
  # nightly release + tag are deleted in ensure-release.sh.
  TAG="nym-vpn-nightly-$(date -u +%Y%m%d%H%M%S)"
fi

echo "::notice:: unified tag: ${TAG}"
{
  echo "core_version=${CORE_VERSION}"
  echo "tag=${TAG}"
} >> "$GITHUB_OUTPUT"
