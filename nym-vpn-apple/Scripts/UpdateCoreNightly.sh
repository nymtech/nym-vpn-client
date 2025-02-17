#!/bin/bash

# Updates the lib and daemon in the iOS+macOS project using nightly builds.
# This script now uses the builds available at:
# https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core/
#
# If no tag is provided as an argument, it defaults to using the 'develop' folder.
#
# It extracts the asset filenames (which include a 14-digit timestamp) and derives:
#   - The library version (e.g. 1.4.0-dev.20250212031000)
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

# -----------------------------------------------------------------------------
# 0. Determine the build tag and the latest timestamp folder to use.
# -----------------------------------------------------------------------------
# Use the first command-line argument if provided, else default to 'develop'
TAG="${1:-develop}"
BASE_URL="https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core"
TAG_URL="${BASE_URL}/${TAG}"

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

# -----------------------------------------------------------------------------
# 1. Extract asset filenames, timestamp, and versions from the release page content.
# -----------------------------------------------------------------------------

ios_asset=$(echo "$release_page_content" | grep -Eo 'nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]{12}_ios_universal\.zip' | head -n 1)
if [[ -z "$ios_asset" ]]; then
    echo "❌ Error: Could not find iOS asset filename in the release page."
    exit 1
fi

TIMESTAMP=$(echo "$ios_asset" | grep -Eo '[0-9]{12}')
if [[ -z "$TIMESTAMP" ]]; then
    echo "❌ Error: Could not extract timestamp from iOS asset filename."
    exit 1
fi

LIB_VERSION=$(echo "$ios_asset" | sed -E 's/nym-vpn-core-v([0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]{12})_ios_universal\.zip/\1/')
if [[ -z "$LIB_VERSION" ]]; then
    echo "❌ Error: Could not extract lib version from iOS asset filename."
    exit 1
fi

DAEMON_VERSION=$(echo "$LIB_VERSION" | sed -E 's/-dev\.[0-9]{14}//')
if [[ -z "$DAEMON_VERSION" ]]; then
    echo "❌ Error: Could not derive daemon version from LIB_VERSION."
    exit 1
fi

echo "Extracted LIB version: $LIB_VERSION"
echo "Derived Daemon version: $DAEMON_VERSION"

ios_download_link="${RELEASE_URL}/${ios_asset}"

# -----------------------------------------------------------------------------
# 2. Fetch the iOS checksum from the corresponding .sha256sum file.
# -----------------------------------------------------------------------------
ios_checksum_url="${RELEASE_URL}/${ios_asset}.sha256sum"
echo "Fetching iOS checksum from $ios_checksum_url..."
ios_checksum=$(curl -Ls "$ios_checksum_url" | grep -Eo '[a-f0-9]{64}' | head -n 1)
if [[ -z "$ios_checksum" ]]; then
    echo "❌ Error: Could not extract iOS checksum from $ios_checksum_url"
    exit 1
fi

echo "iOS Download link: $ios_download_link"
echo "iOS Checksum: $ios_checksum"

# -----------------------------------------------------------------------------
# 3. Update Package.swift with the new iOS asset URL and checksum.
# -----------------------------------------------------------------------------
PACKAGE_FILE_PATH="../MixnetLibrary/Package.swift"

if [[ -f "$PACKAGE_FILE_PATH" ]]; then
    sed -i '' "s|url: \".*\"|url: \"$ios_download_link\"|g" "$PACKAGE_FILE_PATH"
    sed -i '' "s|checksum: \".*\"|checksum: \"$ios_checksum\"|g" "$PACKAGE_FILE_PATH"
    echo "Package.swift has been successfully updated with iOS URL and checksum."
else
    echo "❌ Error: Package.swift file not found at $PACKAGE_FILE_PATH"
    exit 1
fi

# -----------------------------------------------------------------------------
# 4. Update the libVersion in AppVersionProvider.swift with LIB_VERSION (with timestamp).
# -----------------------------------------------------------------------------
LIB_VERSION_NO_TIMESTAMP=$(echo "$LIB_VERSION" | sed -E 's/\.[^.]+$//')

app_version_file="../ServicesMutual/Sources/AppVersionProvider/AppVersionProvider.swift"
if [[ -f "$app_version_file" ]]; then
    sed -i '' "s/public static let libVersion = \".*\"/public static let libVersion = \"$LIB_VERSION_NO_TIMESTAMP\"/g" "$app_version_file"
    echo "libVersion updated to $LIB_VERSION_NO_TIMESTAMP in $app_version_file."
else
    echo "❌ Error: AppVersionProvider.swift file not found at $app_version_file"
    exit 1
fi

# -----------------------------------------------------------------------------
# 5. Process macOS asset: extract the asset filename, download and extract it.
# -----------------------------------------------------------------------------
macos_asset=$(echo "$release_page_content" | grep -Eo 'nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]{12}_macos_universal\.tar\.gz' | head -n 1)
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
    echo "Removed old net.nymtech.vpn.helper file."
fi

if [[ -f "${extracted_folder_name}/nym-vpnd" ]]; then
    cp "${extracted_folder_name}/nym-vpnd" "../Daemon/net.nymtech.vpn.helper"
    chmod +x "../Daemon/net.nymtech.vpn.helper"
    echo "nym-vpnd copied and renamed to net.nymtech.vpn.helper successfully."
else
    echo "❌ Error: ${extracted_folder_name}/nym-vpnd not found."
    exit 1
fi

if [[ -d "${extracted_folder_name}/proto" ]]; then
    rm -rf "../ServicesMacOS/Sources/GRPCManager/proto"
    cp -a "${extracted_folder_name}/proto" "../ServicesMacOS/Sources/GRPCManager"
    echo "proto directory has been copied (with all folders and files) to ../ServicesMacOS/Sources/GRPCManager and overwritten."
else
    echo "❌ Error: ${extracted_folder_name}/proto not found."
    exit 1
fi


if [[ -f "$tar_file_name" ]]; then
    echo "Removing downloaded tar.gz file: $tar_file_name"
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

# -----------------------------------------------------------------------------
# 6. Download iOS package, extract it, and copy uniffi/nym_vpn_lib.swift to destination.
# -----------------------------------------------------------------------------
ios_zip_file=$(basename "$ios_download_link")
echo "Downloading iOS package: $ios_zip_file"
curl -LO "$ios_download_link"
echo "iOS package downloaded successfully: $ios_zip_file"

echo "Extracting iOS package..."
# Extract the zip file in the current directory.
unzip -q "$ios_zip_file"
echo "✅ iOS package extracted successfully."

# Identify the extracted folder.
# Assumes the zip file creates a top-level directory matching the pattern nym-vpn-core-v*ios_universal.
extracted_folder=$(find . -maxdepth 1 -type d -name "nym-vpn-core-v*ios_universal" | head -n 1 | sed 's|^\./||')
if [[ -z "$extracted_folder" ]]; then
    echo "❌ Error: Could not find the extracted folder for the iOS package."
    exit 1
fi

# Define the source Swift file and destination path.
source_swift_file="${extracted_folder}/uniffi/nym_vpn_lib.swift"
destination_swift_path="../MixnetLibrary/Sources/MixnetLibrary/"

if [[ -f "$source_swift_file" ]]; then
    cp "$source_swift_file" "$destination_swift_path"
    echo "✅ nym_vpn_lib.swift copied successfully to $destination_swift_path."
else
    echo "❌ Error: nym_vpn_lib.swift not found at $source_swift_file."
    exit 1
fi

# Cleanup the downloaded ZIP file and the extracted folder.
echo "Cleaning up..."
rm -f "$ios_zip_file"
rm -rf "$extracted_folder"
echo "Cleanup completed."

echo "✅ Update completed successfully for nightly build (LIB_VERSION: $LIB_VERSION, Daemon: $DAEMON_VERSION)."
