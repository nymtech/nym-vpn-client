#!/bin/bash

# Updates prebundled servers in the app so that the picker is never empty.
# Example:
#   nym-vpn-apple/Scripts$ sh UpdatePrebundledServers.sh
# Must be run from nym-vpn-apple/Scripts.

set -euo pipefail
trap 'echo "Error occurred in script at line: ${LINENO}. Exiting." >&2; exit 1' ERR

EXIT_SERVERS_URL="https://nymvpn.com/api/public/v1/directory/gateways/exit"
EXIT_SERVERS_FILE_NAME="../NymVPN/Resources/gatewaysExit.json"

ENTRY_SERVERS_URL="https://nymvpn.com/api/public/v1/directory/gateways/entry"
ENTRY_SERVERS_FILE_NAME="../NymVPN/Resources/gatewaysEntry.json"

VPN_SERVERS_URL="https://nymvpn.com/api/public/v1/directory/gateways?show_vpn_only=true"
VPN_SERVERS_FILE_NAME="../NymVPN/Resources/gatewaysVpn.json"

mkdir -p "$(dirname "$EXIT_SERVERS_FILE_NAME")"

fetch_json() {
  local url="$1" out="$2"
  local tmp="$(mktemp)"
  local hdr="$(mktemp)"

  curl -sS --fail -L --http1.1 \
       -H 'Accept: application/json' \
       -H 'Accept-Encoding: identity, gzip, deflate, br' \
       -D "$hdr" -o "$tmp" "$url"

  local enc
  enc=$(grep -i '^Content-Encoding:' "$hdr" | awk '{print tolower($2)}' | tr -d '\r')
  enc=${enc:-identity}
  echo "↪  $url"
  echo "   Content-Encoding: $enc"

  if [[ "$enc" == "br" ]]; then
    node -e '
      const fs = require("fs"), zlib = require("zlib");
      const [inFile, outFile] = process.argv.slice(1);
      const buf = fs.readFileSync(inFile);
      const out = zlib.brotliDecompressSync(buf);
      fs.writeFileSync(outFile, out);
    ' "$tmp" "${tmp}.dec"
    mv "${tmp}.dec" "$tmp"
  fi

  # Fail if empty
  if [[ ! -s "$tmp" ]]; then
    echo "❌ Error: $url returned an empty file" >&2
    exit 1
  fi

  # Quick sanity: should start with { or [
  local firstchar
  firstchar=$(head -c1 "$tmp")
  if [[ "$firstchar" != "{" && "$firstchar" != "[" ]]; then
    echo "❌ Error: $url did not return JSON (first char: '$firstchar')" >&2
    exit 1
  fi

  # Validate full JSON
  if ! jq -e . "$tmp" >/dev/null 2>&1; then
    echo "❌ Error: $url did not return valid JSON" >&2
    exit 1
  fi

  mv "$tmp" "$out"
  rm -f "$hdr"
}

fetch_json "$EXIT_SERVERS_URL"  "$EXIT_SERVERS_FILE_NAME"
fetch_json "$ENTRY_SERVERS_URL" "$ENTRY_SERVERS_FILE_NAME"
fetch_json "$VPN_SERVERS_URL"   "$VPN_SERVERS_FILE_NAME"

echo "✅ 🇨🇭 🇩🇪 🇫🇷  Prebundled servers updated successfully"
