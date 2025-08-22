#!/bin/bash

# Updates the lib and daemon in the iOS+macOS project
# Example:
# nym-vpn-apple/Scripts$ ./Scripts/UpdateCoreBranch.sh
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

START_TIME=$(date +%s)

# nym-vpn-apple
cd .. 
# nym-vpn-client
cd ..
make build-wireguard-ios
make build-wireguard
cd nym-vpn-core

make build-vpn-lib-swift
make generate-uniffi-ios
make build-mac

# nym-vpn-client
cd ..

# iOS package
SOURCE_PATH="nym-vpn-core/crates/nym-vpn-lib/NymVpnLib/RustFramework.xcframework"
DESTINATION_PATH="nym-vpn-apple/MixnetLibrary/Sources/RustFramework.xcframework"

if [ -e "$SOURCE_PATH" ]; then
    cp -R "$SOURCE_PATH/" "$DESTINATION_PATH"
    echo "✅ RustFramework.xcframework has been successfully copied to $DESTINATION_PATH"
else
    echo "❌ Error: $SOURCE_PATH does not exist. Copy operation failed." >&2
    exit 1
fi

SOURCE_PATH="nym-vpn-core/crates/nym-vpn-lib/NymVpnLib/Sources/NymVpnLib/nym_vpn_lib.swift"
DESTINATION_PATH="nym-vpn-apple/MixnetLibrary/Sources/MixnetLibrary/nym_vpn_lib.swift"

if [ -e "$SOURCE_PATH" ]; then
    cp "$SOURCE_PATH" "$DESTINATION_PATH"
    echo "✅ nym_vpn_lib.swift has been successfully copied to $DESTINATION_PATH"
else
    echo "❌ Error: $SOURCE_PATH does not exist. Copy operation failed." >&2
    exit 1
fi

PACKAGE_FILE="nym-vpn-apple/MixnetLibrary/Package.swift"

sed -i '' '29,32s|^//||' "$PACKAGE_FILE"
sed -i '' '24,28s/^/\/\//' "$PACKAGE_FILE"
echo "✅ binary targets updated in $PACKAGE_FILE"

# macOS

SOURCE_PATH="nym-vpn-core/target/release/nym-vpnd"
DESTINATION_PATH="nym-vpn-apple/Daemon/net.nymtech.vpn.helper"

if [ -e "$SOURCE_PATH" ]; then
    cp "$SOURCE_PATH" "$DESTINATION_PATH"
    chmod +x "$DESTINATION_PATH"
    echo "✅ nym-vpnd has been successfully copied to $DESTINATION_PATH"
else
    echo "❌ Error: $SOURCE_PATH does not exist. Copy operation failed." >&2
    exit 1
fi

SOURCE_PATH="proto/nym/vpn.proto"
DESTINATION_PATH="nym-vpn-apple/ServicesMacOS/Sources/GRPCManager/proto/nym/vpn.proto"

if [ -e "$SOURCE_PATH" ]; then    
    cp "$SOURCE_PATH" "$DESTINATION_PATH"
    echo "✅ vpn.proto has been successfully copied to $DESTINATION_PATH"
else
    echo "❌ Error: $SOURCE_PATH does not exist. Copy operation failed." >&2
    exit 1
fi

cd nym-vpn-apple/ServicesMacOS/Sources/GRPCManager/proto/nym
protoc --swift_out=. vpn.proto
protoc --grpc-swift_out=. vpn.proto
echo "✅ vpn.proto swift grpc files generated"

END_TIME=$(date +%s)
ELAPSED_TIME=$((END_TIME - START_TIME))
echo "Time taken: $ELAPSED_TIME seconds"
echo "✅ Updated successfully"
echo "⚠️⚠️⚠️  ********* IMPORTANT ******* ⚠️⚠️⚠️"
echo "⚠️⚠️⚠️  MixnetLibrary/Package.swift ⚠️⚠️⚠️"
echo "⚠️⚠️⚠️     update binary targets    ⚠️⚠️⚠️"
echo "⚠️⚠️⚠️  *************************** ⚠️⚠️⚠️"
