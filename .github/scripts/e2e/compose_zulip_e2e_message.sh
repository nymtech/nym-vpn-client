#!/usr/bin/env bash
# Compose Zulip markdown for the e2e-test notify-zulip job.
# Verbosity: counts + failed/skipped names (no pass name list).
# Artifacts: GitHub artifact-url links only (auth-gated).
#
# Usage:
#   compose_zulip_e2e_message.sh [--report PATH]
#
# Required env:
#   E2E_STATUS E2E_BRANCH E2E_SHA E2E_SOURCE_LINE E2E_EVENT_NAME E2E_RUN_URL
# Optional env:
#   E2E_SCHEDULE_CRON E2E_TEST_REPORT_URL E2E_DAEMON_LOGS_URL
set -euo pipefail

# Stay under Zulip send-message 10000-byte content limit.
MAX_BYTES=9000

PASS_MARK=$'\xe2\x9c\x85'   # ✅
FAIL_MARK=$'\xe2\x9d\x8c'   # ❌
SKIP_MARK=$'\xe2\x86\xa9\xef\xb8\x8f' # ↪️

REPORT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --report)
      REPORT="${2:-}"
      shift 2
      ;;
    -h | --help)
      echo "usage: $0 [--report PATH]" >&2
      exit 2
      ;;
    *)
      echo "usage: $0 [--report PATH]" >&2
      exit 2
      ;;
  esac
done

: "${E2E_STATUS:?E2E_STATUS is required}"
: "${E2E_BRANCH:?E2E_BRANCH is required}"
: "${E2E_SHA:?E2E_SHA is required}"
: "${E2E_SOURCE_LINE:?E2E_SOURCE_LINE is required}"
: "${E2E_EVENT_NAME:?E2E_EVENT_NAME is required}"
: "${E2E_RUN_URL:?E2E_RUN_URL is required}"

passed=0
failed=0
skipped=0
unknown=0
config_name=""
failed_names=()
skipped_names=()

parse_report() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    return 1
  fi

  local line test_name result line_no=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_no=$((line_no + 1))
    if [[ "$line_no" -eq 1 ]]; then
      config_name="$line"
      continue
    fi
    if [[ "$line_no" -eq 2 ]]; then
      continue
    fi
    [[ -z "$line" ]] && continue

    test_name="${line%% *}"
    result="${line#"${test_name}"}"
    result="${result#"${result%%[![:space:]]*}"}"

    case "$result" in
      "$PASS_MARK")
        passed=$((passed + 1))
        ;;
      "$FAIL_MARK")
        failed=$((failed + 1))
        failed_names+=("$test_name")
        ;;
      "$SKIP_MARK")
        skipped=$((skipped + 1))
        skipped_names+=("$test_name")
        ;;
      *)
        unknown=$((unknown + 1))
        ;;
    esac
  done <"$path"
  return 0
}

has_report=false
if [[ -n "$REPORT" ]]; then
  if parse_report "$REPORT"; then
    has_report=true
  fi
fi

# Args: include_skips (true|false), max_failed_names (int, -1 = all)
emit_body() {
  local include_skips="$1"
  local max_failed="$2"
  local platform truncated_fails=false
  local name shown=0 omitted=0

  echo "**E2E** ${E2E_STATUS}"
  echo ""
  echo "- Branch: \`${E2E_BRANCH}\` @ \`${E2E_SHA}\`"
  echo "- Source: ${E2E_SOURCE_LINE}"
  echo "- Event: \`${E2E_EVENT_NAME}\`"
  if [[ "${E2E_EVENT_NAME}" == "schedule" && -n "${E2E_SCHEDULE_CRON:-}" ]]; then
    echo "- cron: \`${E2E_SCHEDULE_CRON}\`"
  fi

  if [[ "$has_report" == "true" ]]; then
    platform="${config_name:-unknown}"
    echo "- Results: ${passed} passed, ${failed} failed, ${skipped} skipped (\`${platform}\`)"
    if [[ "$unknown" -gt 0 ]]; then
      echo "- unknown results: ${unknown}"
    fi
    if [[ "${#failed_names[@]}" -gt 0 ]]; then
      echo "- Failed:"
      for name in "${failed_names[@]}"; do
        if [[ "$max_failed" -ge 0 && "$shown" -ge "$max_failed" ]]; then
          omitted=$((omitted + 1))
          continue
        fi
        echo "  - \`${name}\`"
        shown=$((shown + 1))
      done
      if [[ "$omitted" -gt 0 ]]; then
        echo "  - _…and ${omitted} more (truncated)_"
        truncated_fails=true
      fi
    fi
    if [[ "$include_skips" == "true" && "${#skipped_names[@]}" -gt 0 ]]; then
      echo "- Skipped:"
      for name in "${skipped_names[@]}"; do
        echo "  - \`${name}\`"
      done
    fi
  else
    echo "- results: (no test report)"
  fi

  if [[ -n "${E2E_TEST_REPORT_URL:-}" || -n "${E2E_DAEMON_LOGS_URL:-}" ]]; then
    echo "- artifacts:"
    if [[ -n "${E2E_TEST_REPORT_URL:-}" ]]; then
      echo "  - [test-report](${E2E_TEST_REPORT_URL})"
    fi
    if [[ -n "${E2E_DAEMON_LOGS_URL:-}" ]]; then
      echo "  - [daemon-logs](${E2E_DAEMON_LOGS_URL})"
    fi
  fi

  echo "- run: ${E2E_RUN_URL}"

  if [[ "$include_skips" != "true" || "$truncated_fails" == "true" ]]; then
    echo ""
    echo "_(message truncated to stay under Zulip size limit)_"
  fi
}

msg_file="$(mktemp "${TMPDIR:-/tmp}/zulip-e2e-msg.XXXXXX")"
trap 'rm -f "$msg_file"' EXIT

emit_body true -1 >"$msg_file"
byte_len=$(wc -c <"$msg_file" | tr -d ' ')

if [[ "$byte_len" -gt "$MAX_BYTES" ]]; then
  # Drop skipped names first.
  emit_body false -1 >"$msg_file"
  byte_len=$(wc -c <"$msg_file" | tr -d ' ')
fi

if [[ "$byte_len" -gt "$MAX_BYTES" && "${#failed_names[@]}" -gt 0 ]]; then
  # Binary-search how many failed names fit under MAX_BYTES.
  local_lo=0
  local_hi=${#failed_names[@]}
  local_best=0
  while [[ "$local_lo" -le "$local_hi" ]]; do
    local_mid=$(((local_lo + local_hi) / 2))
    emit_body false "$local_mid" >"$msg_file"
    byte_len=$(wc -c <"$msg_file" | tr -d ' ')
    if [[ "$byte_len" -le "$MAX_BYTES" ]]; then
      local_best=$local_mid
      local_lo=$((local_mid + 1))
    else
      local_hi=$((local_mid - 1))
    fi
  done
  emit_body false "$local_best" >"$msg_file"
  byte_len=$(wc -c <"$msg_file" | tr -d ' ')
fi

if [[ "$byte_len" -gt "$MAX_BYTES" ]]; then
  # Last resort: keep a minimal header + URLs (always under cap for normal inputs).
  {
    echo "**E2E** ${E2E_STATUS}"
    echo ""
    echo "- branch: \`${E2E_BRANCH}\` @ \`${E2E_SHA}\`"
    echo "- source: ${E2E_SOURCE_LINE}"
    echo "- event: \`${E2E_EVENT_NAME}\`"
    if [[ "$has_report" == "true" ]]; then
      echo "- results: ${passed} passed, ${failed} failed, ${skipped} skipped (truncated)"
    else
      echo "- results: (no test report)"
    fi
    if [[ -n "${E2E_TEST_REPORT_URL:-}" || -n "${E2E_DAEMON_LOGS_URL:-}" ]]; then
      echo "- artifacts:"
      if [[ -n "${E2E_TEST_REPORT_URL:-}" ]]; then
        echo "  - [test-report](${E2E_TEST_REPORT_URL})"
      fi
      if [[ -n "${E2E_DAEMON_LOGS_URL:-}" ]]; then
        echo "  - [daemon-logs](${E2E_DAEMON_LOGS_URL})"
      fi
    fi
    echo "- run: ${E2E_RUN_URL}"
    echo ""
    echo "_(message truncated to stay under Zulip size limit)_"
  } >"$msg_file"
fi

cat "$msg_file"
