#!/bin/bash

# Updates the lib and daemon in the iOS+macOS project using nightly builds.
# This script uses the nightly release available at:
# https://github.com/nymtech/nym-vpn-client/releases/tag/nym-vpn-core-nightly
#
# It extracts the asset filenames (which include a 14-digit timestamp) and derives:
#   - The library version (e.g. 1.3.0-dev.20250205030839)
#   - The daemon version (e.g. 1.3.0)
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
# 1. Get release page content and extract asset filenames, timestamp, and versions
# -----------------------------------------------------------------------------

RELEASE_URL="https://github.com/nymtech/nym-vpn-client/releases/tag/nym-vpn-core-nightly"
PACKAGE_FILE_PATH="../MixnetLibrary/Package.swift"

echo "Fetching release page content from $RELEASE_URL..."
release_page_content=$(curl -s "$RELEASE_URL")

ios_asset=$(echo "$release_page_content" | grep -Eo 'nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]{14}_ios_universal\.zip' | head -n 1)
if [[ -z "$ios_asset" ]]; then
    echo "❌ Error: Could not find iOS asset filename in the release page."
    exit 1
fi

TIMESTAMP=$(echo "$ios_asset" | grep -Eo '[0-9]{14}')
if [[ -z "$TIMESTAMP" ]]; then
    echo "❌ Error: Could not extract timestamp from iOS asset filename."
    exit 1
fi

LIB_VERSION=$(echo "$ios_asset" | sed -E 's/nym-vpn-core-v([0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]{14})_ios_universal\.zip/\1/')
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

ios_download_link="https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-nightly/${ios_asset}"
ios_checksum=$(echo "$release_page_content" | grep -E -A 1 "nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]{14}_ios_universal\.zip" | grep -Eo '[a-f0-9]{64}' | head -n 1)
if [[ -z "$ios_checksum" ]]; then
    echo "❌ Error: Could not extract iOS checksum from the release page."
    exit 1
fi

echo "iOS Download link: $ios_download_link"
echo "iOS Checksum: $ios_checksum"

# -----------------------------------------------------------------------------
# 2. Update Package.swift with the new iOS asset URL and checksum
# -----------------------------------------------------------------------------

if [[ -f "$PACKAGE_FILE_PATH" ]]; then
    sed -i '' "s|url: \".*\"|url: \"$ios_download_link\"|g" "$PACKAGE_FILE_PATH"
    sed -i '' "s|checksum: \".*\"|checksum: \"$ios_checksum\"|g" "$PACKAGE_FILE_PATH"
    echo "Package.swift has been successfully updated with iOS URL and checksum."
else
    echo "❌ Error: Package.swift file not found at $PACKAGE_FILE_PATH"
    exit 1
fi

# -----------------------------------------------------------------------------
# 3. Update the libVersion in AppVersionProvider.swift with LIB_VERSION (with timestamp)
# -----------------------------------------------------------------------------

app_version_file="../ServicesMutual/Sources/AppVersionProvider/AppVersionProvider.swift"
if [[ -f "$app_version_file" ]]; then
    sed -i '' "s/public static let libVersion = \".*\"/public static let libVersion = \"$LIB_VERSION\"/g" "$app_version_file"
    echo "libVersion updated to $LIB_VERSION in $app_version_file."
else
    echo "❌ Error: AppVersionProvider.swift file not found at $app_version_file."
    exit 1
fi

# -----------------------------------------------------------------------------
# 4. Process macOS asset: extract the asset filename, download and extract it.
# -----------------------------------------------------------------------------

macos_asset=$(echo "$release_page_content" | grep -Eo 'nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]{14}_macos_universal\.tar\.gz' | head -n 1)
if [[ -z "$macos_asset" ]]; then
    echo "❌ Error: Could not find macOS asset filename in the release page."
    exit 1
fi

macos_download_link="https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-nightly/${macos_asset}"
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
# 5. Download, extract, and build the source tar.gz for the nightly release
# -----------------------------------------------------------------------------

tar_file_url="https://github.com/nymtech/nym-vpn-client/archive/refs/tags/nym-vpn-core-nightly.tar.gz"
source_tar_file=$(basename "$tar_file_url")
curl -LO "$tar_file_url"
echo "Source tar file downloaded successfully: $source_tar_file"
tar -xzf "$source_tar_file"
echo "✅ Source tar file extracted successfully."
source_folder=$(tar -tf "$source_tar_file" | head -n 1 | cut -f1 -d"/")

cd "${source_folder}" || exit 1
make build-wireguard-ios
cd "nym-vpn-core" || exit 1
make build-vpn-lib-swift
make generate-uniffi-ios
cd ..
cd ..
echo "✅ Makefile executed successfully."
rm -f "$source_tar_file"
echo "✅ Cleaned up the source tar file."

# -----------------------------------------------------------------------------
# 6. Copy generated Swift file from the source to the MixnetLibrary
# -----------------------------------------------------------------------------

source_swift_file="${source_folder}/nym-vpn-core/crates/nym-vpn-lib/uniffi/nym_vpn_lib.swift"
destination_swift_path="../MixnetLibrary/Sources/MixnetLibrary/"

if [[ -f "$source_swift_file" ]]; then
    cp "$source_swift_file" "$destination_swift_path"
    echo "✅ nym_vpn_lib.swift copied successfully to $destination_swift_path."
else
    echo "❌ Error: nym_vpn_lib.swift not found at $source_swift_file."
    exit 1
fi

# -----------------------------------------------------------------------------
# 7. Process proto files: generate Swift gRPC files and copy them to the destination
# -----------------------------------------------------------------------------

proto_folder="${source_folder}/proto/nym"
destination_proto_folder="../../../../ServicesMacOS/Sources/GRPCManager/proto/nym"

cd "$proto_folder"
echo "✅ Changed directory to $proto_folder"
protoc --swift_out=. vpn.proto
echo "✅ vpn.pb.swift generated successfully."
protoc --grpc-swift_out=. vpn.proto
echo "✅ vpn.grpc.swift generated successfully."
mkdir -p "$destination_proto_folder"
cp vpn.grpc.swift vpn.pb.swift vpn.proto "$destination_proto_folder"
echo "✅ Files copied successfully to $destination_proto_folder."
cd -

# -----------------------------------------------------------------------------
# 8. Update daemon Info.plist and final cleanup
# -----------------------------------------------------------------------------

sh UpdateDaemonInfoPlist.sh "$DAEMON_VERSION"

if [[ -d "$source_folder" ]]; then
    rm -rf "$source_folder"
    echo "✅ Cleaned up extracted source folder: $source_folder"
else
    echo "❌ Extracted source folder not found: $source_folder"
fi

echo "✅ Update completed successfully for nightly build (LIB_VERSION: $LIB_VERSION, Daemon: $DAEMON_VERSION)."
