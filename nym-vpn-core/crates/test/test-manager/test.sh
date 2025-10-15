#!/bin/bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -ex

RED='\e[31m'
GREEN='\e[32m'
YELLOW='\e[33m'
BLUE='\e[34m'
NC='\e[0m'

export RUST_LOG=debug
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_FRAMEWORK_ROOT="$(realpath "$SCRIPT_DIR/..")"
CARGO_WORKSPACE_ROOT="$(realpath "${TEST_FRAMEWORK_ROOT}/../..")"
OUTPUT_DIR="${TEST_FRAMEWORK_ROOT}/target/x86_64-unknown-linux-gnu/release"
PACKAGE_DIR="${HOME}/.cache/nym-test/packages"
QCOW_IMAGE="$HOME/iso_images/debian12_cli.qcow2"

# vm_config="first-config"
vm_config="debian12-config"
echo "TEST_FRAMEWORK_ROOT=${TEST_FRAMEWORK_ROOT}"


function help() {
    cargo run -- config vm set --help
}

function list() {
    cargo run -- config vm list
}

function configure() {
    cargo run -- config vm set \
        ${vm_config} \
        "qemu" \
        "${QCOW_IMAGE}" \
        "linux" \
        --package-type "deb" \
        --architecture "x64" \
        --provisioner "ssh" \
        --ssh-user "test" \
        --ssh-password "test" \
        --vcpus 4 \
        --memory 4096

    list
}

function run_vm() {
    cargo run \
        -- run-vm ${vm_config} \
        --vnc 5901
        # --keep-changes
}

function build_deps() {
    # needs to be build within a containerized environment
    # NOT raw like this
    # cargo build --release \
    #     -p test-runner \
    #     -p connection-checker
    pushd $TEST_FRAMEWORK_ROOT
    pwd
    ./scripts/container-run.sh ./scripts/build/test-runner.sh linux

    cp "${OUTPUT_DIR}/connection-checker" "${PACKAGE_DIR}/"
    cp "${OUTPUT_DIR}/test-runner" "${PACKAGE_DIR}/"
    popd
}

function build_nym_deps() {
    pushd ${CARGO_WORKSPACE_ROOT}
    echo -e "======== ${YELLOW} Building Nym deps${NC} ========"
    cargo build --package nym-vpnc --release
    cargo build --package nym-vpnd --release
    echo -e "======== ${GREEN} Finished building Nym deps ${NC} ========"

    cp ./target/release/nym-vpnc ${PACKAGE_DIR}
    cp ./target/release/nym-vpnd ${PACKAGE_DIR}

    popd
}

function run_tests() {
    build_deps
    build_nym_deps

    pushd "${TEST_FRAMEWORK_ROOT}"
    cargo run \
        -p test-manager \
        -- run-tests \
        --vm ${vm_config} \
        --vnc 5901 \
        --nym-mnemonic "${MAINNET_MNEMONIC}" \
        --package-dir "${PACKAGE_DIR}" \
        --runner-dir "$OUTPUT_DIR"\
        --verbose \
        "basic_functionality"
    popd
}


# configure
# run_vm
run_tests
