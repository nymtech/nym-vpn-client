#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RC="${SCRIPT_DIR}/release-core.sh"
failures=0

assert_eq() {
  local label="$1" expected="$2" actual="$3"
  if [[ "$expected" != "$actual" ]]; then
    echo "FAIL ${label}: expected '${expected}', got '${actual}'"
    failures=$((failures + 1))
  else
    echo "ok   ${label}"
  fi
}

assert_fails() {
  local label="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "FAIL ${label}: expected non-zero exit"
    failures=$((failures + 1))
  else
    echo "ok   ${label}"
  fi
}

assert_ok() {
  local label="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "ok   ${label}"
  else
    echo "FAIL ${label}: expected zero exit"
    failures=$((failures + 1))
  fi
}

out="$("$RC" archive-name "nym-vpn-v2026.12.0-beta.1")"
assert_eq "beta version" "version=2026.12.0-beta.1" "$(echo "$out" | sed -n '1p')"
assert_eq "beta archive" \
  "archive=nym-vpn-core-v2026.12.0-beta.1_linux_x86_64.tar.gz" \
  "$(echo "$out" | sed -n '2p')"
assert_eq "beta archive_dir" \
  "archive_dir=nym-vpn-core-v2026.12.0-beta.1_linux_x86_64" \
  "$(echo "$out" | sed -n '3p')"

out="$("$RC" archive-name "nym-vpn-v2026.12.0")"
assert_eq "ship archive" \
  "archive=nym-vpn-core-v2026.12.0_linux_x86_64.tar.gz" \
  "$(echo "$out" | sed -n '2p')"

out="$("$RC" archive-name "nym-vpn-v2026.12.0-nightly.20260729")"
assert_eq "nightly archive" \
  "archive=nym-vpn-core-v2026.12.0-nightly.20260729_linux_x86_64.tar.gz" \
  "$(echo "$out" | sed -n '2p')"

assert_fails "rejects empty tag" "$RC" archive-name
assert_fails "rejects empty version after prefix" "$RC" archive-name "nym-vpn-v"
assert_fails "rejects non-unified tag" "$RC" archive-name "v2026.12.0-beta.1"
assert_fails "rejects bare version" "$RC" archive-name "2026.12.0-beta.1"
assert_fails "rejects path traversal tag" "$RC" archive-name "nym-vpn-v../../etc/passwd"
# Intentional single quotes: assert the tag is rejected, not expanded by the shell.
# shellcheck disable=SC2016
assert_fails "rejects shell metacharacters in tag" \
  "$RC" archive-name 'nym-vpn-v1.0.0-beta.1$(touch /tmp/x)'

assert_ok "validate-tag accepts ship" "$RC" validate-tag "nym-vpn-v2026.12.0"
assert_ok "validate-tag accepts beta" "$RC" validate-tag "nym-vpn-v2026.12.0-beta.1"
assert_fails "validate-tag rejects prefix-only glob abuse" \
  "$RC" validate-tag "nym-vpn-v../../something"

fixture='[
  {"tagName":"nym-vpn-v2026.11.3-beta.3","publishedAt":"2026-07-20T10:00:00Z"},
  {"tagName":"nym-vpn-v2026.12.0-beta.1","publishedAt":"2026-07-28T12:00:00Z"},
  {"tagName":"nym-vpn-v2026.12.0-nightly.20260729","publishedAt":"2026-07-29T04:00:00Z"},
  {"tagName":"nym-vpn-v2026.12.0","publishedAt":"2026-07-30T08:00:00Z"},
  {"tagName":"nym-vpn-v2026.11.3-beta.2","publishedAt":"2026-07-18T10:00:00Z"}
]'
assert_eq "picks newest beta by publishedAt" \
  "nym-vpn-v2026.12.0-beta.1" \
  "$("$RC" pick-beta <<<"$fixture")"
assert_eq "empty when no betas" \
  "" \
  "$("$RC" pick-beta <<<'[{"tagName":"nym-vpn-v2026.12.0","publishedAt":"2026-07-30T08:00:00Z"}]')"
assert_eq "ignores null tagName" \
  "nym-vpn-v2026.12.0-beta.1" \
  "$("$RC" pick-beta <<<'[{"tagName":null,"publishedAt":"2026-07-31T00:00:00Z"},{"tagName":"nym-vpn-v2026.12.0-beta.1","publishedAt":"2026-07-28T12:00:00Z"}]')"
assert_eq "empty when only null tagName" \
  "" \
  "$("$RC" pick-beta <<<'[{"tagName":null,"publishedAt":"2026-07-31T00:00:00Z"}]')"

if [[ "$failures" -ne 0 ]]; then
  echo "${failures} failure(s)"
  exit 1
fi
echo "all tests passed"
