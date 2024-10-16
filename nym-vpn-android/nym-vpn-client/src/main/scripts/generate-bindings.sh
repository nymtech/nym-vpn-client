#!/bin/bash
(cd $PWD/../../nym-vpn-core; cargo run --bin uniffi-bindgen generate --library $PWD/../nym-vpn-android/nym-vpn-client/src/main/jniLibs/arm64-v8a/libnym_vpn_lib.so  --language kotlin --out-dir $PWD/../nym-vpn-android/nym-vpn-client/src/main/java/net/nymtech/vpn -n)
