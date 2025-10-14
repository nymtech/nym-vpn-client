#!/usr/bin/env bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_FRAMEWORK_ROOT="$(realpath $SCRIPT_DIR/../..)"
REPO_DIR="$(realpath $TEST_FRAMEWORK_ROOT/../../..)"
echo "TEST_FRAMEWORK_ROOT=${TEST_FRAMEWORK_ROOT}"
echo "REPO_DIR=${REPO_DIR}"

pushd "$TEST_FRAMEWORK_ROOT"

# shellcheck disable=SC1091
source "$REPO_DIR/scripts/utils/log"

case ${1-:""} in
    linux)
        TARGET=x86_64-unknown-linux-gnu
        shift
    ;;
    windows)
        TARGET=x86_64-pc-windows-gnu
        shift
    ;;
    macos)
        # TODO: x86
        TARGET=aarch64-apple-darwin
        shift
    ;;
    *)
        log_error "Invalid platform. Specify a valid platform as first argument"
        exit 1
esac

cargo build \
    --package test-runner \
    # TODO dz to be removed
    --package connection-checker \
    --release --target "${TARGET}"

# Only build runner image for Windows
if [[ $TARGET == x86_64-pc-windows-gnu ]]; then
    TARGET="$TARGET" ./runner-image.sh
fi

popd

while [[ "$#" -gt 0 ]]; do
    case $1 in
        # Optionally move binaries to some known location
        --output)
            ARTIFACTS_DIR="$TEST_FRAMEWORK_ROOT/target/$TARGET/release"
            mv -t "$1" "$ARTIFACTS_DIR/test-runner" "$ARTIFACTS_DIR/connection-checker"
            ;;
        *)
            log_error "Unknown parameter: $1"
            exit 1
            ;;
    esac
    shift
done
