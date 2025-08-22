#!/bin/bash

# Updates the lib and daemon in the iOS+macOS project using nightly builds.
# This script now uses the builds available at:
# https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core/
#
# If no tag is provided as an argument, it defaults to using the 'develop' folder.
# If a tag is provided, it uses the release folder with that tag.
#
# It extracts the asset filenames (which include a 14-digit timestamp) and derives:
#   - The library version (e.g. 1.4.0-dev.20250212031000 or 1.4.0-beta.202502251100)
#   - The daemon version (e.g. 1.4.0)
#
# Must be run from nym-vpn-apple/Scripts.

# Global error handling
set -e
set -u
set -o pipefail
set -E

error_handler() {
    echo "Error occurred in script at line: ${1}. Exiting."
    exit 1
}
trap 'error_handler $LINENO' ERR
BASE_URL="https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core"

# -----------------------------------------------------------------------------
# 0. Determine the build tag from the current branch.
#    - If branch starts with "release/", use that as TAG.
#    - Otherwise use "develop".
# -----------------------------------------------------------------------------

current_branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"

if [[ "$current_branch" =~ ^release/ ]]; then
  TAG="$current_branch"
  TAG_URL="${BASE_URL}/${TAG}"
  # release builds may have no -dev/-beta timestamp suffix
  macos_pattern='nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+(-(?:dev|beta)\.[0-9]{12})?_macos_universal\.tar\.gz'
else
  TAG="develop"
  TAG_URL="${BASE_URL}/${TAG}"
  # nightly/dev builds include -dev/-beta + 12-digit timestamp
  macos_pattern='nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+-(?:dev|beta)\.[0-9]{12}_macos_universal\.tar\.gz'
fi

echo "Using build tag: ${TAG}"
echo "Fetching folder listing from $TAG_URL..."
# Use -L to follow redirects.
folder_listing=$(curl -Ls "$TAG_URL")

# Extract directories with 12-digit names (e.g. 202502241842/)
latest_folder=$(echo "$folder_listing" | grep -Eo '[0-9]{12}/' | tr -d '/' | sort | tail -n 1)
if [[ -z "$latest_folder" ]]; then
    echo "❌ Error: Could not determine the latest timestamp folder from $TAG_URL"
    exit 1
fi

echo "Latest timestamp folder: $latest_folder"
RELEASE_URL="${TAG_URL}/${latest_folder}"

echo "Fetching release page content from $RELEASE_URL..."
release_page_content=$(curl -Ls "$RELEASE_URL")
if [[ -z "$release_page_content" ]]; then
    echo "❌ Error: Release page content is empty. Please verify that the URL $RELEASE_URL exists and is accessible."
    exit 1
fi

# -----------------------------------------------------------------------------
# 1. Process macOS asset: extract the asset filename, download and extract it.
# -----------------------------------------------------------------------------
macos_asset=$(echo "$release_page_content" | grep -Eo "$macos_pattern" | head -n 1)
if [[ -z "$macos_asset" ]]; then
    echo "❌ Error: Could not find macOS asset filename in the release page."
    exit 1
fi

macos_download_link="${RELEASE_URL}/${macos_asset}"
echo "macOS Download link: $macos_download_link"
curl -LO "$macos_download_link"
echo "macOS file downloaded successfully: $(basename "$macos_download_link")"

tar_file_name=$(basename "$macos_download_link")
tar -xzf "$tar_file_name"
echo "✅ macOS file extracted successfully: $tar_file_name"

extracted_folder_name=$(tar -tf "$tar_file_name" | head -n 1 | cut -f1 -d"/")
if [[ -f "../Daemon/net.nymtech.vpn.helper" ]]; then
    rm "../Daemon/net.nymtech.vpn.helper"
    echo "✅ Removed old net.nymtech.vpn.helper file."
fi

if [[ -f "${extracted_folder_name}/nym-vpnd" ]]; then
    cp "${extracted_folder_name}/nym-vpnd" "../Daemon/net.nymtech.vpn.helper"
    chmod +x "../Daemon/net.nymtech.vpn.helper"
    echo "✅ nym-vpnd copied and renamed to net.nymtech.vpn.helper successfully."
else
    echo "❌ Error: ${extracted_folder_name}/nym-vpnd not found."
    exit 1
fi

if [[ -d "${extracted_folder_name}/proto" ]]; then
    rm -rf "../ServicesMacOS/Sources/GRPCManager/proto"
    cp -a "${extracted_folder_name}/proto" "../ServicesMacOS/Sources/GRPCManager"
    echo "✅ proto directory has been copied (with all folders and files) to ../ServicesMacOS/Sources/GRPCManager and overwritten."
else
    echo "❌ Error: ${extracted_folder_name}/proto not found."
    exit 1
fi

if [[ -f "$tar_file_name" ]]; then
    echo "✅ Removing downloaded tar.gz file: $tar_file_name"
    rm -f "$tar_file_name"
    echo "Downloaded tar.gz file removed successfully."
else
    echo "❌ Downloaded tar.gz file not found: $tar_file_name"
fi

if [[ -d "$extracted_folder_name" ]]; then
    echo "Removing extracted folder: $extracted_folder_name"
    rm -rf "$extracted_folder_name"
    echo "Extracted folder removed successfully."
else
    echo "❌ Extracted folder not found: $extracted_folder_name"
fi

echo "✅ Cleanup completed."
