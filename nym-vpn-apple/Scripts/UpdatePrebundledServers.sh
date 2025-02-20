#!/bin/bash

# Updates prebundled countries in the apps, so country picker would never be empty.
# Example:
# nym-vpn-apple/Scripts$ sh UpdatePrebundledCountries.sh
# Must be run from nym-vpn-apple/Scripts.

set -e
set -u
set -o pipefail
set -E

# Error handler function
error_handler() {
    echo "Error occurred in script at line: ${1}. Exiting."
    exit 1
}
trap 'error_handler $LINENO' ERR  # Capture errors and call error_handler

EXIT_SERVERS_URL="https://nymvpn.com/api/public/v1/directory/gateways/exit"
EXIT_SERVERS_FILE_NAME="../NymVPN/Resources/gatewaysExit.json"

ENTRY_SERVERS_URL="https://nymvpn.com/api/public/v1/directory/gateways/entry"
ENTRY_SERVERS_FILE_NAME="../NymVPN/Resources/gatewaysEntry.json"

VPN_SERVERS_URL="https://nymvpn.com/api/public/v1/directory/gateways?show_vpn_only=true"
VPN_SERVERS_FILE_NAME="../NymVPN/Resources/gatewaysVpn.json"

curl $EXIT_SERVERS_URL > $EXIT_SERVERS_FILE_NAME
curl $ENTRY_SERVERS_URL > $ENTRY_SERVERS_FILE_NAME
curl $VPN_SERVERS_URL > $VPN_SERVERS_FILE_NAME

echo "✅ 🇨🇭 🇩🇪 🇫🇷  Prebundled servers updated successfully"
