#!/bin/bash
echo "Building WireGuard dep"
echo "Working dir: $PWD"
echo "NDK_HOME: $1"
export NDK_TOOLCHAIN_DIR="$1/toolchains/llvm/prebuilt/darwin-x86_64/bin"
sh $PWD/src/tools/nym-vpn-client/wireguard/libwg/build-android.sh --android
echo "Building nym-vpn-lib dep"
(cd $PWD/src/tools/nym-vpn-client/nym-vpn-lib; cargo ndk -t armeabi-v7a -t arm64-v8a -t i686-linux-android -t x86_64-linux-android  -o ../../../main/jniLibs build --release)
