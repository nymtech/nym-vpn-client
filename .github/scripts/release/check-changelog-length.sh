#!/usr/bin/env bash
# Fail fast if the release notes about to be published exceed Google's limit.
# fastlane supply uploads the changelog whose filename matches the AAB's
# version code, so this guard checks exactly changelogs/<VERSION_CODE>.txt in
# every locale that has one. Historical changelogs are left alone.
set -euo pipefail

METADATA_DIR="${METADATA_DIR:-fastlane/metadata/android}"
MAX_CHARS="${MAX_CHARS:-500}"
CONSTANTS_FILE="${CONSTANTS_FILE:-nym-vpn-android/buildSrc/src/main/kotlin/Constants.kt}"

if [[ -z "${VERSION_CODE:-}" ]]; then
  if [[ ! -f "$CONSTANTS_FILE" ]]; then
    echo "::error::VERSION_CODE not set and constants file not found: $CONSTANTS_FILE"
    exit 1
  fi
  VERSION_CODE="$(grep -oE 'VERSION_CODE[[:space:]]*=[[:space:]]*[0-9]+' "$CONSTANTS_FILE" | grep -oE '[0-9]+' | head -n1)"
fi

if [[ -z "${VERSION_CODE:-}" ]]; then
  echo "::error::Could not determine VERSION_CODE."
  exit 1
fi

if [[ ! -d "$METADATA_DIR" ]]; then
  echo "::error::Metadata dir not found: $METADATA_DIR"
  exit 1
fi

echo "Checking release notes for version code ${VERSION_CODE} (limit ${MAX_CHARS} chars)..."
fail=0
checked=0
while IFS= read -r -d '' file; do
  checked=$((checked + 1))
  n="$(python3 -c "import sys; print(len(open(sys.argv[1], encoding='utf-8').read().rstrip('\n')))" "$file")"
  if (( n > MAX_CHARS )); then
    echo "::error file=${file}::Release notes are ${n} chars (limit ${MAX_CHARS}): ${file}"
    fail=1
  else
    echo "  ok (${n}): ${file}"
  fi
done < <(find "$METADATA_DIR" -type f -path "*/changelogs/${VERSION_CODE}.txt" -print0)

if (( checked == 0 )); then
  echo "No changelogs/${VERSION_CODE}.txt found under ${METADATA_DIR} (nothing to upload)."
fi
if (( fail == 0 )); then
  echo "OK: all ${checked} release-notes file(s) for ${VERSION_CODE} are <= ${MAX_CHARS} chars."
else
  echo "::error::Release notes for ${VERSION_CODE} exceed the ${MAX_CHARS}-character Play Store limit (see above)."
fi
exit "$fail"
