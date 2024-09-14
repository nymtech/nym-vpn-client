#!/bin/bash

# Check if the version is provided as a command-line argument
if [[ -z "$1" ]]; then
    echo "Error: No version provided. Usage: sh UpdateCore.sh <version>"
    exit 1
fi

VERSION="$1"  # Version passed as an argument (e.g., 0.2.1)
RELEASE_URL="https://github.com/nymtech/nym-vpn-client/releases/tag/nym-vpn-core-v${VERSION}"  # Construct release URL
PACKAGE_FILE_PATH="../MixnetLibrary/Package.swift"  # Path to Package.swift

# Construct the download link using the provided version for the _ios_universal.zip file
ios_download_link="https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v${VERSION}/nym-vpn-core-v${VERSION}_ios_universal.zip"

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
    echo "Error: Could not construct iOS download link or extract checksum."
    exit 1
fi

# Find and download the _macos_universal.tar.gz file
macos_download_link="https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v${VERSION}/nym-vpn-core-v${VERSION}_macos_universal.tar.gz"

echo "macOS Download link: $macos_download_link"

# Download the _macos_universal.tar.gz file using curl
curl -LO "$macos_download_link"

if [[ $? -eq 0 ]]; then
    echo "macOS file downloaded successfully: $(basename "$macos_download_link")"
    
    # Untar the macOS tar.gz file
    tar -xzf "$(basename "$macos_download_link")"
    if [[ $? -eq 0 ]]; then
        echo "macOS file extracted successfully."
        
        # Remove the old net.nymtech.vpn.helper file in ../Daemon folder
        if [[ -f "../Daemon/net.nymtech.vpn.helper" ]]; then
            rm ../Daemon/net.nymtech.vpn.helper
            echo "Removed old net.nymtech.vpn.helper file."
        fi
        
        # Copy nym-vpnd to ../Daemon folder and rename it to net.nymtech.vpn.helper
        if [[ -f "nym-vpn-core-v${VERSION}_macos_universal/nym-vpnd" ]]; then
            cp "nym-vpn-core-v${VERSION}_macos_universal/nym-vpnd" "../Daemon/net.nymtech.vpn.helper"
            if [[ $? -eq 0 ]]; then
                echo "nym-vpnd copied and renamed to net.nymtech.vpn.helper successfully."
            else
                echo "Error: Failed to copy and rename nym-vpnd."
                exit 1
            fi
        else
            echo "Error: nym-vpnd file not found."
            exit 1
        fi
        
        # Remove the downloaded tar.gz file and untarred folder
        rm -f "$(basename "$macos_download_link")"
        rm -rf "nym-vpn-core-v${VERSION}_macos_universal"
        echo "Cleaned up downloaded and extracted files."
    else
        echo "Error: Failed to extract the macOS file."
        exit 1
    fi
else
    echo "Error: Failed to download the macOS file."
    exit 1
fi

# Download the source zip file
source_zip_link="https://github.com/nymtech/nym-vpn-client/archive/refs/tags/nym-vpn-core-v${VERSION}.zip"
curl -LO "$source_zip_link"

if [[ $? -eq 0 ]]; then
    echo "Source zip file downloaded successfully: $(basename "$source_zip_link")"
    
    # Extract the source zip file
    unzip "$(basename "$source_zip_link")"
    if [[ $? -eq 0 ]]; then
        echo "Source zip file extracted successfully."
        
        # Clean up the downloaded source zip file after extraction
        rm -f "$(basename "$source_zip_link")"
        echo "Cleaned up the source zip file."
    else
        echo "Error: Failed to extract the source zip file."
        exit 1
    fi
else
    echo "Error: Failed to download the source zip file."
    exit 1
fi

# Copy and replace nym_vpn_lib.swift in nym-vpn-core/crates/nym-vpn-lib/uniffi to ../MixnetLibrary/Sources/MixnetLibrary
source_swift_file="nym-vpn-client-nym-vpn-core-v${VERSION}/nym-vpn-core/crates/nym-vpn-lib/uniffi/nym_vpn_lib.swift"
destination_swift_path="../MixnetLibrary/Sources/MixnetLibrary/"

if [[ -f "$source_swift_file" ]]; then
    cp "$source_swift_file" "$destination_swift_path"
    if [[ $? -eq 0 ]]; then
        echo "nym_vpn_lib.swift copied successfully to $destination_swift_path."
    else
        echo "Error: Failed to copy nym_vpn_lib.swift."
        exit 1
    fi
else
    echo "Error: nym_vpn_lib.swift file not found at $source_swift_file."
    exit 1
fi

# Run protoc commands in the extracted proto/nym folder
proto_folder="nym-vpn-client-nym-vpn-core-v${VERSION}/proto/nym"
destination_folder="../ServicesMacOS/Sources/GRPCManager/proto/nym"

# Change directory to the proto/nym folder
cd "$proto_folder" || { echo "Error: Failed to change directory to $proto_folder"; exit 1; }
echo pwd

# Run protoc commands to generate swift files
protoc --swift_out=. vpn.proto
if [[ $? -eq 0 ]]; then
    echo "vpn.pb.swift generated successfully."
else
    echo "Error: Failed to generate vpn.pb.swift."
    exit 1
fi

protoc --grpc-swift_out=. vpn.proto
if [[ $? -eq 0 ]]; then
    echo "vpn.grpc.swift generated successfully."
else
    echo "Error: Failed to generate vpn.grpc.swift."
    exit 1
fi

# Copy the generated files and proto file to the correct destination folder
destination_folder="../../../../ServicesMacOS/Sources/GRPCManager/proto/nym"
mkdir -p "$destination_folder"
cp vpn.grpc.swift vpn.pb.swift vpn.proto "$destination_folder"
if [[ $? -eq 0 ]]; then
    echo "Files copied successfully to $destination_folder."
else
    echo "Error: Failed to copy files to $destination_folder."
    exit 1
fi

# Go back to the previous directory
cd - || exit


# Remove the downloaded source zip file and extracted folder
source_zip_file="nym-vpn-core-v${VERSION}.zip"
extracted_folder="nym-vpn-client-nym-vpn-core-v${VERSION}"

# Remove the zip file
rm -f "$source_zip_file"
if [[ $? -eq 0 ]]; then
    echo "Source zip file removed successfully: $source_zip_file."
else
    echo "Error: Failed to remove source zip file."
fi

# Remove the sources extracted folder
rm -rf "$extracted_folder"
if [[ $? -eq 0 ]]; then
    echo "Extracted sources folder removed successfully: $extracted_folder."
else
    echo "Error: Failed to remove extracted folder."
fi

