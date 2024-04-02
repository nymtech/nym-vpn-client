wgpath="$(pwd)/build/lib/universal-apple-darwin"
libPath="$(pwd)/target/aarch64-apple-ios/release/libnym_vpn_lib.a"
RUSTFLAGS="-L build/lib/universal-apple-darwin" cargo build --target aarch64-apple-ios-sim -p nym-vpn-lib --release
RUSTFLAGS="-L build/lib/universal-apple-darwin" cargo build --target aarch64-apple-ios -p nym-vpn-lib --release
RUSTFLAGS="-L $wgpath" cargo run --bin uniffi-bindgen generate --library $libPath --language swift --out-dir uniffi -n

# Rename uniffi output
mv "uniffi/nym_vpn_libFFI.modulemap" "uniffi/module.modulemap"

# TODO: nym_vpn_lib.swift should not be included in headers, we need it in SPM package

# Remove old dir if it exists and create a new one
rm -rf "target/NymVpnLib.xcframework"
mkdir -p "target/NymVpnLib.xcframework"

# Create framework
xcodebuild -create-xcframework \
    -library "./target/aarch64-apple-ios/release/libnym_vpn_lib.a" \
    -headers "./uniffi" \
    -library "./target/aarch64-apple-ios-sim/release/libnym_vpn_lib.a" \
    -headers "./uniffi" \
    -output "./target/NymVpnLib.xcframework"

# Output of create xcframeowork needs to be in package "Libs" folder