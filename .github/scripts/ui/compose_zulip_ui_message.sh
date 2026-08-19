#!/usr/bin/env bash
# Compose Zulip markdown for the ci-maestro-* notify-zulip jobs.
# One message per platform; both workflows call this script.
#
# Verbosity: counts + failed/skipped names (no pass name list).
# Artifacts: GitHub artifact-url links only (auth-gated).
#
# Usage:
#   compose_zulip_ui_message.sh [--report PATH]
#
# Required env:
#   UI_PLATFORM UI_STATUS UI_BRANCH UI_SHA UI_EVENT_NAME UI_RUN_URL
# Optional env:
#   UI_SCHEDULE_CRON UI_REPORT_URL UI_DEBUG_URL
#
# A missing, empty, or unparseable report degrades to a "(no test report)"
# message and still exits 0 - the notification must always be delivered.
set -euo pipefail

# Stay under Zulip send-message 10000-byte content limit.
MAX_BYTES=9000

PASS_MARK=$'\xe2\x9c\x85'             # OK
FAIL_MARK=$'\xe2\x9d\x8c'             # X
SKIP_MARK=$'\xe2\x86\xa9\xef\xb8\x8f' # arrow

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

: "${UI_PLATFORM:?UI_PLATFORM is required}"
: "${UI_STATUS:?UI_STATUS is required}"
: "${UI_BRANCH:?UI_BRANCH is required}"
: "${UI_SHA:?UI_SHA is required}"
: "${UI_EVENT_NAME:?UI_EVENT_NAME is required}"
: "${UI_RUN_URL:?UI_RUN_URL is required}"

case "$UI_PLATFORM" in
  android) PLATFORM_LABEL="Android" ;;
  ios) PLATFORM_LABEL="iOS" ;;
  *) PLATFORM_LABEL="$UI_PLATFORM" ;;
esac

case "$UI_STATUS" in
  passed) STATUS_MARK="$PASS_MARK" ;;
  skipped) STATUS_MARK="$SKIP_MARK" ;;
  *) STATUS_MARK="$FAIL_MARK" ;;
esac

passed=0
failed=0
skipped=0
failed_names=()
skipped_names=()

# Emits "key<TAB>value" lines. Exits non-zero when the report is unusable, so
# the caller falls through to the no-report path.
parse_junit() {
  local path="$1"
  [[ -f "$path" && -s "$path" ]] || return 1
  python3 - "$path" <<'PY'
import sys
import xml.etree.ElementTree as ET

# Text-mode stdout translates "\n" to "\r\n" on Windows hosts, which would leave
# a stray CR inside every parsed value. Pin the line ending and the encoding.
sys.stdout.reconfigure(newline="\n", encoding="utf-8")

try:
    root = ET.parse(sys.argv[1]).getroot()
except Exception:
    sys.exit(1)

# Maestro emits <testsuites> for grouped flows and a bare <testsuite> otherwise.
cases = root.findall(".//testcase")
if not cases:
    sys.exit(1)

passed = failed = skipped = 0
failed_names = []
skipped_names = []

for case in cases:
    name = case.get("name") or case.get("classname") or "unnamed"
    if case.find("skipped") is not None:
        state = "skipped"
    elif case.find("failure") is not None or case.find("error") is not None:
        state = "failed"
    else:
        # No child element: fall back to a status attribute when Maestro sets one.
        status = (case.get("status") or "").strip().upper()
        if status in ("SKIPPED", "IGNORED"):
            state = "skipped"
        elif status in ("FAILED", "ERROR"):
            state = "failed"
        else:
            state = "passed"

    if state == "skipped":
        skipped += 1
        skipped_names.append(name)
    elif state == "failed":
        failed += 1
        failed_names.append(name)
    else:
        passed += 1

out = sys.stdout
out.write("passed\t%d\n" % passed)
out.write("failed\t%d\n" % failed)
out.write("skipped\t%d\n" % skipped)
for name in failed_names:
    out.write("fail\t%s\n" % name.replace("\t", " ").replace("\n", " "))
for name in skipped_names:
    out.write("skip\t%s\n" % name.replace("\t", " ").replace("\n", " "))
PY
}

has_report=false
if [[ -n "$REPORT" ]]; then
  parsed_file="$(mktemp "${TMPDIR:-/tmp}/zulip-ui-parsed.XXXXXX")"
  trap 'rm -f "$parsed_file"' EXIT
  if parse_junit "$REPORT" >"$parsed_file" 2>/dev/null; then
    has_report=true
    while IFS=$'\t' read -r key value; do
      # Belt and braces: tolerate a CR from any python that ignores the above.
      value="${value%$'\r'}"
      case "$key" in
        passed) passed="$value" ;;
        failed) failed="$value" ;;
        skipped) skipped="$value" ;;
        fail) failed_names+=("$value") ;;
        skip) skipped_names+=("$value") ;;
      esac
    done <"$parsed_file"
  fi
fi

emit_artifact_links() {
  if [[ -n "${UI_REPORT_URL:-}" || -n "${UI_DEBUG_URL:-}" ]]; then
    echo "- artifacts:"
    if [[ -n "${UI_REPORT_URL:-}" ]]; then
      echo "  - [maestro-${UI_PLATFORM}-report](${UI_REPORT_URL})"
    fi
    if [[ -n "${UI_DEBUG_URL:-}" ]]; then
      echo "  - [maestro-${UI_PLATFORM}-debug](${UI_DEBUG_URL})"
    fi
  fi
}

# Args: include_skips (true|false), max_failed_names (int, -1 = all)
emit_body() {
  local include_skips="$1"
  local max_failed="$2"
  local truncated_fails=false
  local name shown=0 omitted=0

  echo "**UI ${PLATFORM_LABEL}** ${STATUS_MARK} ${UI_STATUS}"
  echo ""
  echo "- Branch: \`${UI_BRANCH}\` @ \`${UI_SHA}\`"
  echo "- Event: \`${UI_EVENT_NAME}\`"
  if [[ "${UI_EVENT_NAME}" == "schedule" && -n "${UI_SCHEDULE_CRON:-}" ]]; then
    echo "- cron: \`${UI_SCHEDULE_CRON}\`"
  fi

  if [[ "$has_report" == "true" ]]; then
    echo "- Results: ${passed} passed, ${failed} failed, ${skipped} skipped"
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
        echo "  - _...and ${omitted} more (truncated)_"
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
    echo "- Results: (no test report)"
  fi

  emit_artifact_links

  echo "- run: ${UI_RUN_URL}"

  if [[ "$include_skips" != "true" || "$truncated_fails" == "true" ]]; then
    echo ""
    echo "_(message truncated to stay under Zulip size limit)_"
  fi
}

msg_file="$(mktemp "${TMPDIR:-/tmp}/zulip-ui-msg.XXXXXX")"
trap 'rm -f "$msg_file" "${parsed_file:-}"' EXIT

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
  # Last resort: minimal header + URLs (always under cap for normal inputs).
  {
    echo "**UI ${PLATFORM_LABEL}** ${STATUS_MARK} ${UI_STATUS}"
    echo ""
    echo "- Branch: \`${UI_BRANCH}\` @ \`${UI_SHA}\`"
    echo "- Event: \`${UI_EVENT_NAME}\`"
    if [[ "$has_report" == "true" ]]; then
      echo "- Results: ${passed} passed, ${failed} failed, ${skipped} skipped (truncated)"
    else
      echo "- Results: (no test report)"
    fi
    emit_artifact_links
    echo "- run: ${UI_RUN_URL}"
    echo ""
    echo "_(message truncated to stay under Zulip size limit)_"
  } >"$msg_file"
fi

cat "$msg_file"
