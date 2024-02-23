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
bash $PWD/src/tools/nym-vpn-client/wireguard/libwg/build-android.sh
echo "Building nym-vpn-lib dep"
(cd $PWD/src/tools/nym-vpn-client/nym-vpn-lib; cargo ndk -t armeabi-v7a -t arm64-v8a -t i686-linux-android -t x86_64-linux-android  -o ../../../main/jniLibs build --release)
#mv wireguard
mv $PWD/src/tools/nym-vpn-client/build/lib/extraJni/arm64-v8a/libwg.so $PWD/src/main/jniLibs/arm64-v8a/
mv $PWD/src/tools/nym-vpn-client/build/lib/extraJni/armeabi-v7a/libwg.so $PWD/src/main/jniLibs/armeabi-v7a/
mv $PWD/src/tools/nym-vpn-client/build/lib/extraJni/x86/libwg.so $PWD/src/main/jniLibs/x86/
mv $PWD/src/tools/nym-vpn-client/build/lib/extraJni/x86_64/libwg.so $PWD/src/main/jniLibs/x86_64/