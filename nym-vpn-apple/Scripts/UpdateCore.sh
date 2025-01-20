#!/bin/bash

# Updates the lib and daemon in the iOS+macOS project
# Example:
# nym-vpn-apple/Scripts$ ./Scripts/UpdateCore.sh 0.2.1
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
    echo "❌ Error: No version provided. Usage: sh UpdateCore.sh <version>"
    exit 1
fi

VERSION="$1"  # Version passed as an argument (e.g., 1.0.0-dev-apple)

# Extract the base version by removing any suffix after "dev" if it exists
BASE_VERSION=$(echo "$VERSION" | sed -E 's/(.*dev)[^ ]*/\1/')

# Construct the release URL
RELEASE_URL="https://github.com/nymtech/nym-vpn-client/releases/tag/nym-vpn-core-v${VERSION}"  # Release URL using original version
PACKAGE_FILE_PATH="../MixnetLibrary/Package.swift"  # Path to Package.swift

# Construct the iOS download link using the provided version
ios_download_link="https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v${VERSION}/$(curl -s "$RELEASE_URL" | grep 'ios_universal.zip' | awk '{print $2}' | head -n 1)"

# Fetch the release page content
release_page_content=$(curl -s "$RELEASE_URL")

# Extract the checksum for the _ios_universal.zip file
ios_checksum=$(echo "$release_page_content" | grep -A 1 "_ios_universal.zip" | grep -o '[a-f0-9]\{64\}' | head -n 1)

# Print the constructed iOS download link and checksum
echo "iOS Download link: $ios_download_link"
echo "iOS Checksum: $ios_checksum"

# Replace only the URL and checksum in the .binaryTarget block for _ios_universal.zip
if [[ -n "$ios_download_link" && -n "$ios_checksum" ]]; then
    if [[ -f "$PACKAGE_FILE_PATH" ]]; then
        # Replace the URL in the .binaryTarget block
        sed -i '' "s|url: \".*_ios_universal.zip\"|url: \"$ios_download_link\"|g" "$PACKAGE_FILE_PATH"

        # Replace the checksum in the .binaryTarget block
        sed -i '' "s|checksum: \".*\"|checksum: \"$ios_checksum\"|g" "$PACKAGE_FILE_PATH"

        echo "Package.swift has been successfully updated with iOS url and checksum."
    else
        echo "Error: Package.swift file not found at $PACKAGE_FILE_PATH"
        exit 1
    fi
else
    echo "❌ Error: Could not construct iOS download link or extract checksum."
    exit 1
fi

# Update libVersion in AppVersionProvider.swift to the new version
app_version_file="../ServicesMutual/Sources/AppVersionProvider/AppVersionProvider.swift"
if [[ -f "$app_version_file" ]]; then
    sed -i '' "s/public static let libVersion = \".*\"/public static let libVersion = \"$VERSION\"/g" "$app_version_file"
    echo "libVersion updated to $VERSION in $app_version_file."
else
    echo "❌ Error: AppVersionProvider.swift file not found at $app_version_file."
    exit 1
fi

# Construct the macOS download link using the extracted base version
macos_download_link="https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v${VERSION}/$(curl -s "$RELEASE_URL" | grep 'macos_universal.tar.gz' | awk '{print $7}' | head -n 1)"

echo "macOS Download link: $macos_download_link"

# Download the _macos_universal.tar.gz file using curl
curl -LO "$macos_download_link"
echo "macOS file downloaded successfully: $(basename "$macos_download_link")"

# Untar the macOS tar.gz file
tar_file_name=$(basename "$macos_download_link")
tar -xzf "$tar_file_name"
echo "✅ macOS file extracted successfully. $tar_file_name"

# Determine the extracted folder name
extracted_folder_name=$(tar -tf "$tar_file_name" | head -n 1 | cut -f1 -d"/")

# Remove the old net.nymtech.vpn.helper file in ../Daemon folder
if [[ -f "../Daemon/net.nymtech.vpn.helper" ]]; then
    rm ../Daemon/net.nymtech.vpn.helper
    echo "Removed old net.nymtech.vpn.helper file."
fi

# Copy nym-vpnd to ../Daemon folder and rename it to net.nymtech.vpn.helper
if [[ -f "${extracted_folder_name}/nym-vpnd" ]]; then
    cp "${extracted_folder_name}/nym-vpnd" "../Daemon/net.nymtech.vpn.helper"
    chmod +x "../Daemon/net.nymtech.vpn.helper"
    echo "nym-vpnd copied and renamed to net.nymtech.vpn.helper successfully."
else
    echo "❌ Error: ${extracted_folder_name}/nym-vpnd not found."
fi

# Remove the downloaded tar.gz file
tar_file_name=$(basename "$macos_download_link")

if [[ -f "$tar_file_name" ]]; then
    echo "Removing downloaded tar.gz file: $tar_file_name"
    rm -f "$tar_file_name"
    echo "Downloaded tar.gz file removed successfully."
else
    echo " ❌Downloaded tar.gz file not found: $tar_file_name"
fi

# Remove the extracted folder
if [[ -d "$extracted_folder_name" ]]; then
    echo "Removing extracted folder: $extracted_folder_name"
    rm -rf "$extracted_folder_name"
    echo "Extracted folder removed successfully."
else
    echo "❌ Extracted folder not found: $extracted_folder_name"
fi

# Download the source zip file
tar_file_url="https://github.com/nymtech/nym-vpn-client/archive/refs/tags/nym-vpn-core-v${VERSION}.tar.gz"
tar_file_name=$(basename "$tar_file_url")

# Download the tar.gz file using curl
curl -LO "$tar_file_url"
echo "Source tar file downloaded successfully: $tar_file_name"

# Extract the source tar.gz file
tar -xzf "$tar_file_name"
echo "✅ Source tar file extracted successfully."

# Determine the extracted folder name
# Assuming the unzipped directory follows the pattern 'nym-vpn-client-*'
extracted_folder_name=$(tar -tf "$tar_file_name" | head -n 1 | cut -f1 -d"/")

# Generate uniffy

cd "${extracted_folder_name}" || exit 1
make build-wireguard-ios

cd "nym-vpn-core" || exit 1
make build-vpn-lib-swift
make generate-uniffi-ios
cd ..
cd ..
echo "✅ Makefile executed successfully."


# Clean up the downloaded source tar file after extraction
rm -f "$(basename "$tar_file_name")"
echo "✅ Cleaned up the source ztar file."

# Copy and replace nym_vpn_lib.swift in nym-vpn-core/crates/nym-vpn-lib/uniffi to ../MixnetLibrary/Sources/MixnetLibrary
source_swift_file="nym-vpn-client-nym-vpn-core-v${VERSION}/nym-vpn-core/crates/nym-vpn-lib/uniffi/nym_vpn_lib.swift"
destination_swift_path="../MixnetLibrary/Sources/MixnetLibrary/"

cp "$source_swift_file" "$destination_swift_path"
echo "✅ nym_vpn_lib.swift copied successfully to $destination_swift_path."

# Run protoc commands in the extracted proto/nym folder
proto_folder="nym-vpn-client-nym-vpn-core-v${VERSION}/proto/nym"
destination_folder="../ServicesMacOS/Sources/GRPCManager/proto/nym"

# Change directory to the proto/nym folder
cd "$proto_folder"
echo "✅ Changed directory to $proto_folder"

# Run protoc commands to generate swift files
protoc --swift_out=. vpn.proto
echo "✅ vpn.pb.swift generated successfully."

protoc --grpc-swift_out=. vpn.proto
echo "✅ vpn.grpc.swift generated successfully."

# Copy the generated files and proto file to the correct destination folder
destination_folder="../../../../ServicesMacOS/Sources/GRPCManager/proto/nym"
mkdir -p "$destination_folder"
cp vpn.grpc.swift vpn.pb.swift vpn.proto "$destination_folder"
echo "✅ Files copied successfully to $destination_folder."

# Go back to the previous directory
cd -

# Update daemon Info.plist
sh UpdateDaemonInfoPlist.sh ${VERSION}

# Remove the downloaded source zip file and extracted folder
rm -f "nym-vpn-core-v${VERSION}.zip"
rm -rf "nym-vpn-client-nym-vpn-core-v${VERSION}"
echo "✅ Cleaned up downloaded and extracted files."
echo "✅ Updated successfully"
