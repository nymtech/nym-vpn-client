#!/usr/bin/env bash
echo "Building dependencies"
echo "NDK_HOME: $NDK_PATH"
source "$HOME/.cargo/env"
export PATH="$PATH:$HOME/go/bin"
export NDK_TOOLCHAIN_DIR="$NDK_PATH/toolchains/llvm/prebuilt/$(basename $NDK_PATH/toolchains/llvm/prebuilt/*/)/bin"

# install cargo dependencies
cargo install cargo-ndk cargo-license --locked

export RUSTFLAGS="-L ../build/lib/aarch64-linux-android -L ../build/lib/x86_64-unknown-linux-gnu"

bash ../wireguard/build-wireguard-go.sh
bash ../wireguard/libwg/build-android.sh

(cd ../nym-vpn-core/crates/nym-vpn-lib; cargo ndk -t arm64-v8a -o ../../../nym-vpn-android/nym-vpn-client/src/main/jniLibs build --release)
(cd ../nym-vpn-core; cargo run --bin uniffi-bindgen generate --library ./target/aarch64-linux-android/release/libnym_vpn_lib.so  --language kotlin --out-dir ../nym-vpn-android/nym-vpn-client/src/main/java/net/nymtech/vpn -n)
cargo license -j --avoid-dev-deps --current-dir ../nym-vpn-core/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > ./nym-vpn-client/src/main/assets/licenses_rust.json
mv ../android/app/build/extraJni/arm64-v8a/libwg.so nym-vpn-client/src/main/jniLibs/arm64-v8a/
