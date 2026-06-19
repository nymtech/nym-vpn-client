#!/usr/bin/env bash
# Apply a precomputed nightly version to a Cargo.toml.
#
# The version is computed ONCE upstream (release-all-platforms.yml -> resolve-version.sh)
# and passed down to every build/publish job, so the binary's embedded version and the
# release/artifact names all share the same timestamp (no per-job drift). This script is a
# dumb applier: it never computes a version itself.
#
# No-op when NIGHTLY_VERSION is empty — that is the ship/legacy path, where the real
# Cargo.toml version is kept as-is.
#
# Inputs (env):
#   NIGHTLY_VERSION  precomputed version string (empty = leave Cargo.toml untouched)
#   CARGO_TOML       path to the Cargo.toml to patch
#   VERSION_KEY      toml key to set (workspace.package.version | package.version)
set -euo pipefail

if [[ -z "${NIGHTLY_VERSION:-}" ]]; then
  echo "::notice:: no nightly version provided; leaving ${CARGO_TOML:-Cargo.toml} unchanged"
  exit 0
fi

: "${CARGO_TOML:?CARGO_TOML is required when NIGHTLY_VERSION is set}"
: "${VERSION_KEY:?VERSION_KEY is required when NIGHTLY_VERSION is set}"

echo "::notice:: set ${VERSION_KEY} = ${NIGHTLY_VERSION} in ${CARGO_TOML}"
toml set "$CARGO_TOML" "$VERSION_KEY" "$NIGHTLY_VERSION" > "${CARGO_TOML}.patched"
mv "${CARGO_TOML}.patched" "$CARGO_TOML"
