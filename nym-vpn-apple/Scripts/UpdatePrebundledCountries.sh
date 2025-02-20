#!/bin/bash

# Updates prebundled countries in the apps, so country picker would never be empty.
# Example:
# nym-vpn-apple/Scripts$ sh UpdatePrebundledCountries.sh
# Must be run from nym-vpn-apple/Scripts.

# Global error handling
set -e  # Exit immediately on non-zero status return
set -u  # Treat unset variables as errors
set -o pipefail  # Exit on the first error in a pipeline
set -E

# Error handler function
error_handler() {
    echo "Error occurred in script at line: ${1}. Exiting."
    exit 1
}
trap 'error_handler $LINENO' ERR  # Capture errors and call error_handler

EXIT_COUNTRIES_URL="https://nymvpn.com/api/public/v1/directory/gateways/exit/countries"
EXIT_COUNTRIES_FILE_NAME="../NymVPN/Resources/gatewaysExitCountries.json"

ENTRY_COUNTRIES_URL="https://nymvpn.com/api/public/v1/directory/gateways/entry/countries"
ENTRY_COUNTRIES_FILE_NAME="../NymVPN/Resources/gatewaysEntryCountries.json"

VPN_COUNTRIES_URL="https://nymvpn.com/api/public/v1/directory/gateways/countries?show_vpn_only=true"
VPN_COUNTRIES_FILE_NAME="../NymVPN/Resources/gatewaysVpnCountries.json"

curl $EXIT_COUNTRIES_URL > $EXIT_COUNTRIES_FILE_NAME
curl $ENTRY_COUNTRIES_URL > $ENTRY_COUNTRIES_FILE_NAME
curl $VPN_COUNTRIES_URL > $VPN_COUNTRIES_FILE_NAME

echo "✅ 🇨🇭 🇩🇪 🇫🇷  Prebundled countries updated successfully"
