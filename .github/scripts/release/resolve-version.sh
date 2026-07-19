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
#   channel  | guard                 | version          | tag
#   ---------+-----------------------+------------------+------------------------------------
#   nightly  | -                     | <base>-nightly   | nym-vpn-v<base>-nightly.YYYYMMDD
#   beta     | auto +1 from releases | <base>-beta.<N>  | nym-vpn-v<base>-beta.<N>
#   ship     | full MUST NOT -beta   | <full>           | nym-vpn-v<full>
#
# <full> = workspace version verbatim; <base> = major.minor.patch (pre-release stripped).
#
# Beta does NOT read its -beta.N from Cargo.toml. The branch pins the BASE
# (humans bump major.minor.patch manually); the suffix auto-advances here. Needs
# GH_TOKEN to list releases and the checkout's `origin` remote to read tags.
set -euo pipefail

CHANNEL="${RELEASE_CHANNEL:?RELEASE_CHANNEL required (nightly|beta|ship)}"
CARGO_ENTRY="${CARGO_ENTRY:-nym-vpn-core/Cargo.toml}"

FULL="$(cargo-get workspace.package.version --entry "$CARGO_ENTRY")"
BASE="${FULL%%-*}"   # strip the first '-' and everything after → major.minor.patch
echo "::notice:: channel=${CHANNEL} cargo_version=${FULL} base=${BASE}"

case "$CHANNEL" in
  nightly)
    NIGHTLY_TIMESTAMP="$(date -u +%Y%m%d)"
    VERSION="${BASE}-nightly.${NIGHTLY_TIMESTAMP}"
    # Date-only tag (cron runs once/day). Native immutable releases forbid REUSING a
    # published tag name, so a manual same-day re-run can collide — accepted.
    TAG="nym-vpn-v${BASE}-nightly.${NIGHTLY_TIMESTAMP}"
    ;;
  beta)
    # Auto-advance the -beta.N suffix off the BASE pinned by the branch. Find the
    # highest beta already cut for this base from BOTH sources, then add 1:
    #   - published beta RELEASES (the canonical "current version from releases")
    #   - left-behind beta TAGS (ship deletes beta releases but KEEPS their tags;
    #     those tags are immutable, so we must never recompute onto one)
    # The bumped version is stamped into the build in-runner by the apply-version
    # composite and is never committed back to the branch.
    BASE_RE="${BASE//./[.]}"
    HIGHEST=0

    while IFS= read -r n; do
      [ -n "$n" ] || continue
      [ "$n" -gt "$HIGHEST" ] && HIGHEST="$n"
    done < <(gh release list --limit 200 --json tagName \
      -q ".[] | .tagName | select(test(\"^nym-vpn-v${BASE_RE}-beta[.][0-9]+\$\"))" \
      | sed -E "s/^nym-vpn-v${BASE}-beta[.]//")

    while IFS= read -r n; do
      [ -n "$n" ] || continue
      [ "$n" -gt "$HIGHEST" ] && HIGHEST="$n"
    done < <(git ls-remote --tags origin "refs/tags/nym-vpn-v${BASE}-beta.*" 2>/dev/null \
      | sed -E "s#.*refs/tags/nym-vpn-v${BASE}-beta[.]([0-9]+)\$#\1#" \
      | grep -E '^[0-9]+$' || true)

    NEXT=$(( HIGHEST + 1 ))
    VERSION="${BASE}-beta.${NEXT}"
    TAG="nym-vpn-v${BASE}-beta.${NEXT}"
    echo "::notice:: highest existing beta for ${BASE}=${HIGHEST} → next beta.${NEXT}"
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

if [ -n "$NIGHTLY_TIMESTAMP" ]; then
  echo "nightly_timestamp=${NIGHTLY_TIMESTAMP}" >> "$GITHUB_OUTPUT"
fi
