#!/bin/bash

# Example:
# nym-vpn-apple/Scripts$ ./Scripts/UpdateDaemonInfoPlist.sh 0.2.1
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

# Check if the version is provided as a command-line argument
if [[ -z "${1:-}" ]]; then
    echo "Error: No version provided. Usage: sh UpdateCore.sh <version>"
    exit 1
fi

VERSION="$1"  # Version passed as an argument (e.g., 0.2.1)

# Path to the Info.plist file
PLIST_PATH="../Daemon/Info.plist"

# Check if the plist file exists
if [[ ! -f "$PLIST_PATH" ]]; then
    echo "Error: Info.plist not found at $PLIST_PATH"
    exit 1
fi

# Get the current CFBundleVersion value
CURRENT_BUNDLE_VERSION=$(/usr/libexec/PlistBuddy -c "Print CFBundleVersion" "$PLIST_PATH")

# Increment the CFBundleVersion by 1
NEW_BUNDLE_VERSION=$((CURRENT_BUNDLE_VERSION + 1))

# Update CFBundleShortVersionString with the new version
/usr/libexec/PlistBuddy -c "Set CFBundleShortVersionString $VERSION" "$PLIST_PATH"

# Update CFBundleVersion with the incremented version
/usr/libexec/PlistBuddy -c "Set CFBundleVersion $NEW_BUNDLE_VERSION" "$PLIST_PATH"

echo "Updated CFBundleShortVersionString to $VERSION"
echo "Updated CFBundleVersion to $NEW_BUNDLE_VERSION"