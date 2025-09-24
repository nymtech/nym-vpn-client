#!/bin/bash

# Updates prebundled countries in the apps, so country picker would never be empty.
# Example:
# nym-vpn-apple/Scripts$ sh UpdatePrebundledCountries.sh
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

  jq -e . "$tmp" >/dev/null
  mv "$tmp" "$out"
  rm -f "$hdr"
}

fetch_json "$EXIT_SERVERS_URL"  "$EXIT_SERVERS_FILE_NAME"
fetch_json "$ENTRY_SERVERS_URL" "$ENTRY_SERVERS_FILE_NAME"
fetch_json "$VPN_SERVERS_URL"   "$VPN_SERVERS_FILE_NAME"

echo "✅ 🇨🇭 🇩🇪 🇫🇷  Prebundled servers updated successfully"
