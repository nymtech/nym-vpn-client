#!/bin/bash
# requires cargo, cargo-ndk and android NDK to be installed
echo "Building WireGuard dep"
echo "Working dir: $PWD"
echo "NDK_HOME: $1"
#fix to work with different OS
archDir=$(basename $1/toolchains/llvm/prebuilt/*/)
echo "archdir: ${archDir}"
export ANDROID_NDK_HOME="$1"
export NDK_TOOLCHAIN_DIR="$1/toolchains/llvm/prebuilt/${archDir}/bin"
bash $PWD/src/tools/nym-vpn-client/wireguard/build-wireguard-go.sh
bash $PWD/src/tools/nym-vpn-client/wireguard/libwg/build-android.sh
echo "Building nym-vpn-lib dep"
(cd $PWD/src/tools/nym-vpn-client/nym-vpn-lib; cargo ndk -t armeabi-v7a -t arm64-v8a -t i686-linux-android -t x86_64-linux-android  -o ../../../main/jniLibs build --release; cargo ndk -t aarch64-linux-android build)
#mv wireguard

case  "$(uname -s)" in
    Darwin*) export RUSTFLAGS="-L ${PWD}/src/tools/nym-vpn-client/build/lib/universal-apple-darwin";;
    Linux*) export RUSTFLAGS="-L ${PWD}/src/tools/nym-vpn-client/build/lib/x86_64-unknown-linux-gnu";;
    MINGW*|MSYS_NT*) export RUSTFLAGS="-L ${PWD}/src/tools/nym-vpn-client/build/lib/x86_64-pc-windows-msvc";;
esac

(cd $PWD/src/tools/nym-vpn-client; cargo run --bin uniffi-bindgen generate --library ./target/aarch64-linux-android/debug/libnym_vpn_lib.so  --language kotlin --out-dir ../../main/java/net/nymtech/vpn -n)
#fix package name
sed -i 's/package nym-vpn-lib;/package nym_vpn_lib;/g' $PWD/src/main/java/net/nymtech/vpn/nym-vpn-lib/nym_vpn_lib.kt

mv $PWD/src/main/jniLibs/arm64-v8a/libnym_vpn_lib.so $PWD/src/main/jniLibs/arm64-v8a/libnym_vpn_lib.so
mv $PWD/src/main/jniLibs/armeabi-v7a/libnym_vpn_lib.so $PWD/src/main/jniLibs/armeabi-v7a/libnym_vpn_lib.so
mv $PWD/src/main/jniLibs/x86/libnym_vpn_lib.so $PWD/src/main/jniLibs/x86/libnym_vpn_lib.so
mv $PWD/src/main/jniLibs/x86_64/libnym_vpn_lib.so $PWD/src/main/jniLibs/x86_64/libnym_vpn_lib.so

mv $PWD/src/tools/nym-vpn-client/android/app/build/extraJni/arm64-v8a/libwg.so $PWD/src/main/jniLibs/arm64-v8a/
mv $PWD/src/tools/nym-vpn-client/android/app/build/extraJni/armeabi-v7a/libwg.so $PWD/src/main/jniLibs/armeabi-v7a/
mv $PWD/src/tools/nym-vpn-client/android/app/build/extraJni/x86/libwg.so $PWD/src/main/jniLibs/x86/
mv $PWD/src/tools/nym-vpn-client/android/app/build/extraJni/x86_64/libwg.so $PWD/src/main/jniLibs/x86_64/