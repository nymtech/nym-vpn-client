#!/usr/bin/env bash
# Resolve the release channel, the unified version, and the GH tag.
# Inputs (env):
#   RELEASE_CHANNEL  nightly | beta | ship   (required)
#   CARGO_ENTRY      path to the core workspace Cargo.toml (default nym-vpn-core/Cargo.toml)
# Outputs (GITHUB_OUTPUT):
#   channel   echoed channel
#   version   single version stamped into the core lib AND every platform build + the tag
#   tag       unified GitHub release tag
#
#   channel  | guard                | version          | tag
#   ---------+----------------------+------------------+------------------------------------
#   nightly  | -                    | <base>-nightly   | nym-vpn-v<base>-nightly.YYYYMMDD
#   beta     | full MUST have -beta  | <full>           | nym-vpn-v<full>
#   ship     | full MUST NOT -beta   | <full>           | nym-vpn-v<full>
#
# <full> = workspace version verbatim; <base> = major.minor.patch (pre-release stripped).
set -euo pipefail

CHANNEL="${RELEASE_CHANNEL:?RELEASE_CHANNEL required (nightly|beta|ship)}"
CARGO_ENTRY="${CARGO_ENTRY:-nym-vpn-core/Cargo.toml}"

FULL="$(cargo-get workspace.package.version --entry "$CARGO_ENTRY")"
BASE="${FULL%%-*}"   # strip the first '-' and everything after → major.minor.patch
echo "::notice:: channel=${CHANNEL} cargo_version=${FULL} base=${BASE}"

case "$CHANNEL" in
  nightly)
    VERSION="${BASE}-nightly"
    # Date-only tag (cron runs once/day). Native immutable releases forbid REUSING a
    # published tag name, so a manual same-day re-run can collide — accepted.
    TAG="nym-vpn-v${BASE}-nightly.$(date -u +%Y%m%d)"
    ;;
  beta)
    if [[ "$FULL" != *-beta* ]]; then
      echo "::error:: beta channel requires a -beta core version, got: ${FULL}"
      exit 1
    fi
    VERSION="${FULL}"
    TAG="nym-vpn-v${FULL}"
    ;;
  ship)
    if [[ "$FULL" == *-beta* ]]; then
      echo "::error:: refusing to ship a beta version: ${FULL}"
      exit 1
    fi
    VERSION="${FULL}"
    TAG="nym-vpn-v${FULL}"
    ;;
  *)
    echo "::error:: unknown release channel: ${CHANNEL} (expected nightly|beta|ship)"
    exit 1
    ;;
esac

echo "::notice:: version=${VERSION} tag=${TAG}"
{
  echo "channel=${CHANNEL}"
  echo "version=${VERSION}"
  echo "tag=${TAG}"
} >> "$GITHUB_OUTPUT"
