#!/bin/bash

set -euo pipefail

# requires cargo, go, cargo-ndk and android NDK to be installed
echo "Working dir: $PWD"
echo "NDK_HOME: $1"

#fix to work with different OS
archDir=$(basename $1/toolchains/llvm/prebuilt/*/)
echo "archdir: ${archDir}"

export ANDROID_NDK_HOME="$1"
export NDK_TOOLCHAIN_DIR="$1/toolchains/llvm/prebuilt/${archDir}/bin"

# for reproducibility
if [[ "$OSTYPE" == "linux-gnu"* ]]; then

    PROJECT_ROOT="$(realpath $(pwd)/../..)"

    # Rust compiler sysroot that typically points to ~/.rustup/toolchains/<toolchain>
    RUST_COMPILER_SYS_ROOT=$(rustc --print sysroot)

    # Rust flags used in reproducible builds to replace common paths with defaults
    IDEMPOTENT_RUSTFLAGS="-C link-args=-Wl,--build-id=none --remap-path-prefix \
                ${HOME}=~ --remap-path-prefix ${PROJECT_ROOT}=/buildroot --remap-path-prefix \
                ${RUST_COMPILER_SYS_ROOT}=/sysroot"

    # Export rust flags enforcing reproducible builds
    export RUSTFLAGS=$IDEMPOTENT_RUSTFLAGS
    # Tell build tools to use unix timestamp of zero as a reference date
    export SOURCE_DATE_EPOCH=0
    # Force vergen to emit stable values
    export VERGEN_IDEMPOTENT=1

    export VERGEN_GIT_BRANCH="VERGEN_IDEMPOTENT_OUTPUT"
fi

cd ../..

bash wireguard/build-wireguard-go.sh --android
pushd nym-vpn-core/crates/nym-vpn-lib

cargo install cargo-ndk cargo-license --locked

cargo ndk -t arm64-v8a -o ../../../nym-vpn-android/core/src/main/jniLibs build --release
popd

pushd nym-vpn-core

cargo run --bin uniffi-bindgen generate --library ./target/aarch64-linux-android/release/libnym_vpn_lib.so  --language kotlin --out-dir ../nym-vpn-android/core/src/main/java/net/nymtech/vpn -n
popd

mv nym-vpn-android/app/build/extraJni/arm64-v8a/libwg.so nym-vpn-android/core/src/main/jniLibs/arm64-v8a/

cargo license -j --avoid-dev-deps --current-dir nym-vpn-core/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > nym-vpn-android/core/src/main/assets/licenses_rust.json
