#!/usr/bin/env bash

script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $script_dir

make build-vpn-lib-swift
cp -R nym-vpn-lib/NymVpnLib/RustFramework.xcframework/ \
  ../nym-vpn-apple/MixnetLibrary/Sources/MixnetLibrary/RustFramework.xcframework
cp nym-vpn-lib/NymVpnLib/Sources/NymVpnLib/nym_vpn_lib.swift \
  ../nym-vpn-apple/MixnetLibrary/Sources/MixnetLibrary/nym_vpn_lib.swift
