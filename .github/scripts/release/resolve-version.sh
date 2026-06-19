#!/usr/bin/env bash
# Resolve the release version + unified tag, and (for nightly) the per-platform nightly
# Cargo versions.
# Inputs (env): SHIP (true|false), CARGO_ENTRY (default nym-vpn-core/Cargo.toml),
#               APP_CARGO_ENTRY (default nym-vpn-app/src-tauri/Cargo.toml),
#               GITHUB_EVENT_NAME (provided by Actions)
# Outputs (GITHUB_OUTPUT): core_version, tag, core_nightly_version, app_nightly_version
#   The two *_nightly_version outputs are empty on ship builds. On nightly builds they
#   share ONE timestamp + label so every platform's build/publish job applies the exact
#   same string (no per-job drift). Each is based on its own project's X.Y.Z so the
#   projects keep their independent version numbers.
set -euo pipefail

SHIP="${SHIP:-false}"
CARGO_ENTRY="${CARGO_ENTRY:-nym-vpn-core/Cargo.toml}"
APP_CARGO_ENTRY="${APP_CARGO_ENTRY:-nym-vpn-app/src-tauri/Cargo.toml}"

CORE_VERSION="$(cargo-get workspace.package.version --entry "$CARGO_ENTRY")"
echo "::notice:: core version: ${CORE_VERSION}"

CORE_NIGHTLY_VERSION=""
APP_NIGHTLY_VERSION=""

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

  if [ "${GITHUB_EVENT_NAME:-}" = "schedule" ]; then NIGHTLY_LABEL="nightly"; else NIGHTLY_LABEL="dev"; fi
  STAMP="$(date -u +%Y%m%d%H%M)"
  CORE_BASE="$(cargo-get workspace.package.version --major --minor --patch --delimiter='.' --entry "$CARGO_ENTRY")"
  APP_BASE="$(cargo-get package.version --major --minor --patch --delimiter='.' --entry "$APP_CARGO_ENTRY")"
  CORE_NIGHTLY_VERSION="${CORE_BASE}-${NIGHTLY_LABEL}.${STAMP}"
  APP_NIGHTLY_VERSION="${APP_BASE}-${NIGHTLY_LABEL}.${STAMP}"
  echo "::notice:: core nightly version: ${CORE_NIGHTLY_VERSION}"
  echo "::notice:: app nightly version: ${APP_NIGHTLY_VERSION}"
fi

echo "::notice:: unified tag: ${TAG}"
{
  echo "core_version=${CORE_VERSION}"
  echo "tag=${TAG}"
  echo "core_nightly_version=${CORE_NIGHTLY_VERSION}"
  echo "app_nightly_version=${APP_NIGHTLY_VERSION}"
} >> "$GITHUB_OUTPUT"
