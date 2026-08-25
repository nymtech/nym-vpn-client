#!/usr/bin/env bash
# Guards install-apple.sh flag contract and no personal device ids.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="$ROOT/install-apple.sh"

bash -n "$SCRIPT"
"$SCRIPT" --help | grep -q -- '--macos'
"$SCRIPT" --help | grep -q -- '--ios'

if "$SCRIPT" --dry-run --skip-core --skip-app >/dev/null 2>&1; then
  printf 'expected --ios/--macos to be required\n' >&2
  exit 1
fi

# Apple hardware UDIDs are 8 hex + hyphen + 16 hex, usually starting 0000.
# Reject any literal of that shape. Do not encode a real device id or name here.
if grep -E '0000[0-9A-Fa-f]{4}-[0-9A-Fa-f]{16}' "$SCRIPT"; then
  printf 'hardcoded Apple hardware UDID must not ship in the installer\n' >&2
  exit 1
fi

printf 'install-apple-selftest: ok\n'
