#!/bin/bash

set -euo pipefail

# requires cargo, cargo-ndk and android NDK to be installed
echo "Building WireGuard dep"
echo "Working dir: $PWD"
echo "NDK_HOME: $1"
#fix to work with different OS
archDir=$(basename $1/toolchains/llvm/prebuilt/*/)
echo "archdir: ${archDir}"
export ANDROID_NDK_HOME="$1"
export NDK_TOOLCHAIN_DIR="$1/toolchains/llvm/prebuilt/${archDir}/bin"
bash $PWD/../../wireguard/build-wireguard-go.sh --android
echo "Building nym-vpn-lib dep"

PROJECT_ROOT="$(realpath $PWD/../..)"
# Rust compiler sysroot that typically points to ~/.rustup/toolchains/<toolchain>
RUST_COMPILER_SYS_ROOT=$(rustc --print sysroot)
# Rust flags used in reproducible builds to replace common paths with defaults
IDEMPOTENT_RUSTFLAGS="-C link-args=-Wl,--build-id=none --remap-path-prefix ${HOME}=~ --remap-path-prefix ${PROJECT_ROOT}=/buildroot --remap-path-prefix ${RUST_COMPILER_SYS_ROOT}=/sysroot"
# Export rust flags enforcing reproducible builds
export RUSTFLAGS=$IDEMPOTENT_RUSTFLAGS
# Tell build tools to use unix timestamp of zero as a reference date
export SOURCE_DATE_EPOCH=0
# Force vergen to emit stable values
export VERGEN_IDEMPOTENT=1

export VERGEN_GIT_BRANCH="VERGEN_IDEMPOTENT_OUTPUT"

#fix emulators later
#(cd $PWD/src/tools/nym-vpn-client/crates/nym-vpn-lib; cargo ndk -t armeabi-v7a -t arm64-v8a -t i686-linux-android -t x86_64-linux-android  -o ../../../main/jniLibs build --release)
pushd $PWD/../../nym-vpn-core/crates/nym-vpn-lib
cargo ndk -t arm64-v8a -o ../../../nym-vpn-android/nym-vpn-client/src/main/jniLibs build --release
popd

pushd $PWD/../../nym-vpn-core
cargo run --bin uniffi-bindgen generate --library ./target/aarch64-linux-android/release/libnym_vpn_lib.so  --language kotlin --out-dir ../nym-vpn-android/nym-vpn-client/src/main/java/net/nymtech/vpn -n
popd

cargo license -j --avoid-dev-deps --current-dir ../../nym-vpn-core/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > ./src/main/assets/licenses_rust.json

mv $PWD/../../nym-vpn-android/app/build/extraJni/arm64-v8a/libwg.so $PWD/src/main/jniLibs/arm64-v8a/
#mv $PWD/src/tools/nym-vpn-client/nym-vpn-android/app/build/extraJni/armeabi-v7a/libwg.so $PWD/src/main/jniLibs/armeabi-v7a/
#mv $PWD/src/tools/nym-vpn-client/nym-vpn-android/app/build/extraJni/x86/libwg.so $PWD/src/main/jniLibs/x86/
#mv $PWD/src/tools/nym-vpn-client/nym-vpn-android/app/build/extraJni/x86_64/libwg.so $PWD/src/main/jniLibs/x86_64/
