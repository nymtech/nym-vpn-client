#!/usr/bin/env bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -eu

CARGO_REGISTRY_VOLUME_NAME=${CARGO_REGISTRY_VOLUME_NAME:-"cargo-registry"}
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
PACKAGE_DIR=${PACKAGE_DIR:-"$HOME/.cache/mullvad-test/packages"}

if [ ! -d "$PACKAGE_DIR" ]; then
  echo "$PACKAGE_DIR does not exist. It is needed to build the test bundle, so please go ahead and create the directory and re-run this script."
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" > /dev/null && pwd )"
echo "SCRIPT_DIR=${SCRIPT_DIR}"
TEST_FRAMEWORK_ROOT="$(realpath "$SCRIPT_DIR/..")"
echo "TEST_FRAMEWORK_ROOT=${TEST_FRAMEWORK_ROOT}"
REPO_DIR="$(realpath $TEST_FRAMEWORK_ROOT/../../..)"
echo "REPO_DIR=${REPO_DIR}"

pushd "$SCRIPT_DIR"

# shellcheck disable=SC1091
source "${REPO_DIR}/scripts/utils/log"

if [[ "$(uname -s)" != "Linux" ]]; then
    log_error "$0 only works on Linux"
    exit 1
fi

container_image=$(cat "${REPO_DIR}/building/linux-container-image.txt")
"$CONTAINER_RUNNER" build -t mullvadvpn-app-tests --build-arg IMAGE="${container_image}" .

popd

exec "$CONTAINER_RUNNER" run --rm -it \
    -v "${CARGO_REGISTRY_VOLUME_NAME}":/root/.cargo/registry:Z \
    -v "${REPO_DIR}":/build:z \
    -w "/build/nym-vpn-core/crates/test" \
    -e CARGO_TARGET_DIR=/build/nym-vpn-core/crates/test/target \
    -v "${PACKAGE_DIR}":/packages:Z \
    -e PACKAGE_DIR=/packages \
    mullvadvpn-app-tests \
    /bin/bash -c "$*"
