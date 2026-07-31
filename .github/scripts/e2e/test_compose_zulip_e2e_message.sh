#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE="${SCRIPT_DIR}/compose_zulip_e2e_message.sh"
failures=0

assert_contains() {
  local label="$1" needle="$2" haystack="$3"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "FAIL ${label}: missing '${needle}'"
    echo "----- output -----"
    echo "$haystack"
    echo "------------------"
    failures=$((failures + 1))
  else
    echo "ok   ${label}"
  fi
}

assert_not_contains() {
  local label="$1" needle="$2" haystack="$3"
  if [[ "$haystack" == *"$needle"* ]]; then
    echo "FAIL ${label}: unexpectedly found '${needle}'"
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

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=$'\xe2\x9c\x85'
FAIL=$'\xe2\x9d\x8c'
SKIP=$'\xe2\x86\xa9\xef\xb8\x8f'

# Matches SummaryLogger layout: config name, JSON Os, then "name emoji" lines.
cat >"${TMP}/report.txt" <<EOF
debian12_cli
"linux"
test_daemon_info ${PASS}
test_dns_leak ${PASS}
test_allow_lan_off_blocks_prior_resolver_tcp ${FAIL}
test_optional_skip ${SKIP}
EOF

export E2E_STATUS="e2e failure"
export E2E_BRANCH="feat/e2e-notify"
export E2E_SHA="217912acd44d7a6a7f1089695a9e5cc837db87f4"
export E2E_SOURCE_LINE='zigbuild from `217912acd44d7a6a7f1089695a9e5cc837db87f4`'
export E2E_EVENT_NAME="workflow_dispatch"
export E2E_RUN_URL="https://github.com/nymtech/nym-vpn-client/actions/runs/30627386088"
export E2E_TEST_REPORT_URL="https://github.com/nymtech/nym-vpn-client/actions/runs/30627386088/artifacts/8793208152"
export E2E_DAEMON_LOGS_URL="https://github.com/nymtech/nym-vpn-client/actions/runs/30627386088/artifacts/8793207461"

out="$("$COMPOSE" --report "${TMP}/report.txt")"

assert_contains "status line" "**E2E** e2e failure" "$out"
assert_contains "branch" "branch: \`feat/e2e-notify\`" "$out"
assert_contains "sha" "217912acd44d7a6a7f1089695a9e5cc837db87f4" "$out"
assert_contains "counts" "2 passed, 1 failed, 1 skipped (\`debian12_cli\`)" "$out"
assert_contains "failed name" "test_allow_lan_off_blocks_prior_resolver_tcp" "$out"
assert_contains "skipped name" "test_optional_skip" "$out"
assert_not_contains "no pass name list" "test_daemon_info" "$out"
assert_contains "test-report link" "[test-report](${E2E_TEST_REPORT_URL})" "$out"
assert_contains "daemon-logs link" "[daemon-logs](${E2E_DAEMON_LOGS_URL})" "$out"
assert_contains "run url" "$E2E_RUN_URL" "$out"

# Missing report file → explicit no-report line; still includes branch/run.
unset E2E_TEST_REPORT_URL E2E_DAEMON_LOGS_URL
out_missing="$("$COMPOSE" --report "${TMP}/does-not-exist.txt")"
assert_contains "missing report" "results: (no test report)" "$out_missing"
assert_contains "missing still has branch" "feat/e2e-notify" "$out_missing"
assert_contains "missing still has run" "$E2E_RUN_URL" "$out_missing"
assert_not_contains "no artifacts section when empty" "artifacts:" "$out_missing"

# Required env missing → non-zero.
assert_fails "requires E2E_STATUS" \
  env -u E2E_STATUS "$COMPOSE" --report "${TMP}/report.txt"

# Truncation keeps failure names and URLs.
export E2E_TEST_REPORT_URL="https://example.test/report"
export E2E_DAEMON_LOGS_URL="https://example.test/logs"
{
  echo "debian12_cli"
  echo '"linux"'
  echo "test_the_only_failure ${FAIL}"
  i=0
  while [[ "$i" -lt 600 ]]; do
    printf 'test_pad_%03d %s\n' "$i" "$SKIP"
    i=$((i + 1))
  done
} >"${TMP}/huge.txt"

out_huge="$("$COMPOSE" --report "${TMP}/huge.txt")"
byte_len=$(printf '%s' "$out_huge" | wc -c | tr -d ' ')
if [[ "$byte_len" -gt 10000 ]]; then
  echo "FAIL truncation: output is ${byte_len} bytes (>10000)"
  failures=$((failures + 1))
else
  echo "ok   truncation under 10000 bytes (${byte_len})"
fi
assert_contains "truncation keeps failure" "test_the_only_failure" "$out_huge"
assert_contains "truncation keeps report url" "https://example.test/report" "$out_huge"
assert_contains "truncation marker" "truncated" "$out_huge"

# Many failures must also stay under Zulip's 10KB content cap.
{
  echo "debian12_cli"
  echo '"linux"'
  i=0
  while [[ "$i" -lt 500 ]]; do
    printf 'test_fail_with_a_longer_name_%03d %s\n' "$i" "$FAIL"
    i=$((i + 1))
  done
} >"${TMP}/many-fail.txt"

out_many="$("$COMPOSE" --report "${TMP}/many-fail.txt")"
many_len=$(printf '%s' "$out_many" | wc -c | tr -d ' ')
if [[ "$many_len" -gt 10000 ]]; then
  echo "FAIL many-failure truncation: output is ${many_len} bytes (>10000)"
  failures=$((failures + 1))
else
  echo "ok   many-failure under 10000 bytes (${many_len})"
fi
assert_contains "many-failure keeps a fail name" "test_fail_with_a_longer_name_" "$out_many"
assert_contains "many-failure keeps run url" "$E2E_RUN_URL" "$out_many"
assert_contains "many-failure keeps report url" "https://example.test/report" "$out_many"

if [[ "$failures" -ne 0 ]]; then
  echo "${failures} failure(s)"
  exit 1
fi
echo "all tests passed"
