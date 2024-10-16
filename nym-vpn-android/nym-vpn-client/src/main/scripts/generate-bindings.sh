#!/bin/bash
cargo run --bin uniffi-bindgen generate --library ./src/main/jniLibs/arm64-v8a/libnym_vpn_lib.so  --language kotlin --out-dir ./src/main/java/net/nymtech/vpn -n
