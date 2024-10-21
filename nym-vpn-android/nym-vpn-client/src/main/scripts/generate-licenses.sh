#!/bin/bash
cargo license -j --avoid-dev-deps --current-dir ../../nym-vpn-core/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > ./src/main/assets/licenses_rust.json
