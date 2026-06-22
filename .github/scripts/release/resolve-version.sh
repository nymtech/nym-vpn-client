#!/usr/bin/env bash
# Resolve the release version + unified tag, and (for nightly) the shared nightly version.
# Inputs (env): SHIP (true|false), CARGO_ENTRY (default nym-vpn-core/Cargo.toml),
#               GITHUB_EVENT_NAME (provided by Actions)
# Outputs (GITHUB_OUTPUT): core_version, tag, nightly_version
#   nightly_version is empty on ship builds. On nightly builds EVERY platform shares this
#   ONE string (core X.Y.Z + label + timestamp): this pipeline builds nightlies from
#   develop, so all platforms carry the exact same version derived from core, with no
#   per-platform version drift.
set -euo pipefail

SHIP="${SHIP:-false}"
CARGO_ENTRY="${CARGO_ENTRY:-nym-vpn-core/Cargo.toml}"

CORE_VERSION="$(cargo-get workspace.package.version --entry "$CARGO_ENTRY")"
echo "::notice:: core version: ${CORE_VERSION}"

NIGHTLY_VERSION=""

if [[ "$SHIP" == "true" ]]; then
  if [[ "$CORE_VERSION" == *beta* ]]; then
    echo "::error:: refusing to ship a beta version: ${CORE_VERSION}" >&2
    exit 1
  fi
  TAG="nym-vpn-v${CORE_VERSION}"
else
  # Unique-per-run nightly tag. Native immutable releases forbid REUSING a tag
  # once it was published, so each nightly gets a fresh timestamp; the prior
  # nightly release + tag are deleted in ensure-release.sh.
  TAG="nym-vpn-nightly-$(date -u +%Y%m%d%H%M%S)"

  if [[ "${GITHUB_EVENT_NAME:-}" == "schedule" ]]; then NIGHTLY_LABEL="nightly"; else NIGHTLY_LABEL="dev"; fi
  STAMP="$(date -u +%Y%m%d%H%M)"
  CORE_BASE="$(cargo-get workspace.package.version --major --minor --patch --delimiter='.' --entry "$CARGO_ENTRY")"
  NIGHTLY_VERSION="${CORE_BASE}-${NIGHTLY_LABEL}.${STAMP}"
  echo "::notice:: nightly version (all platforms): ${NIGHTLY_VERSION}"
fi

echo "::notice:: unified tag: ${TAG}"
{
  echo "core_version=${CORE_VERSION}"
  echo "tag=${TAG}"
  echo "nightly_version=${NIGHTLY_VERSION}"
} >> "$GITHUB_OUTPUT"
