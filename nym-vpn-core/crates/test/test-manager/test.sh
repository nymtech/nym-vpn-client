#!/bin/bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -e

GREEN='\e[32m'
YELLOW='\e[33m'
BLUE='\e[34m'
NC='\e[0m'

export RUST_LOG=debug
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_FRAMEWORK_ROOT="$(realpath "$SCRIPT_DIR/..")"
CARGO_WORKSPACE_ROOT="$(realpath "${TEST_FRAMEWORK_ROOT}/../..")"
REPO_DIR="$(realpath "${TEST_FRAMEWORK_ROOT}/../../..")"
OUTPUT_DIR="${TEST_FRAMEWORK_ROOT}/target/x86_64-unknown-linux-gnu/release"
PACKAGE_DIR="${PACKAGE_DIR:-${HOME}/.cache/nym-test/packages}"
CONTAINER_RUNNER="${CONTAINER_RUNNER:-podman}"
CARGO_REGISTRY_VOLUME_NAME="${CARGO_REGISTRY_VOLUME_NAME:-cargo-registry}"
IMAGE_TAG="nym-test-builder"

# Default values (can be overridden by env vars or CLI params)
NYM_TEST_QCOW_IMAGE="${NYM_TEST_QCOW_IMAGE:-}"
NYM_TEST_VM_CONFIG="${NYM_TEST_VM_CONFIG:-}"


echo "TEST_FRAMEWORK_ROOT=${TEST_FRAMEWORK_ROOT}"

function usage() {
    echo "Usage: $0 [options] <command>"
    echo ""
    echo "Available commands:"
    echo "  configure   - Configure a new VM"
    echo "  list        - List existing configurations"
    echo "  run-tests   - Build dependencies and run tests"
    echo "  run-vm      - Run a VM without tests for you to connect to"
    echo ""
}

function check_required_vars() {
    local missing=0
    local qcow_image_var_name="NYM_TEST_QCOW_IMAGE"
    local vm_config_var_name="NYM_TEST_VM_CONFIG"

    if [ -z "$NYM_TEST_QCOW_IMAGE" ]; then
        echo "Error: ${qcow_image_var_name} is not set"
        echo "  Set it via environment variable: export ${qcow_image_var_name}=/path/to/image.qcow2"
        missing=1
    fi

    if [ -z "$NYM_TEST_VM_CONFIG" ]; then
        echo "Error: ${vm_config_var_name} is not set"
        echo "  Set it via environment variable: export ${vm_config_var_name}=config-name"
        missing=1
    fi

    if [ $missing -eq 1 ]; then
        echo ""
        usage
        exit 1
    fi


    echo -e "${GREEN}Using VM configuration:${NC}"
    echo -e "\tQCOW Image: ${BLUE}${NYM_TEST_QCOW_IMAGE}${NC}"
    echo -e "\tVM Config:  ${BLUE}${NYM_TEST_VM_CONFIG}${NC}"
    echo ""
}

function help() {
    cargo run -- config vm set --help
}

function list() {
    cargo run -- config vm list
}

function configure() {
    cargo run -- config vm set \
        ${NYM_TEST_VM_CONFIG} \
        "qemu" \
        "${NYM_TEST_QCOW_IMAGE}" \
        "linux" \
        --package-type "deb" \
        --architecture "x64" \
        --provisioner "ssh" \
        --ssh-user "test" \
        --ssh-password "test" \
        --vcpus 2 \
        --memory 1024

    list
}

function run_vm() {
    cargo run \
        -- run-vm ${NYM_TEST_VM_CONFIG} \
        --vnc 5901
        # --keep-changes
}

function build_all_in_container() {
    mkdir -p "${PACKAGE_DIR}"

    echo -e "======== ${YELLOW} Building container image${NC} ========"
    "$CONTAINER_RUNNER" build -t "${IMAGE_TAG}" -f "${TEST_FRAMEWORK_ROOT}/Dockerfile" "${REPO_DIR}"
    echo -e "======== ${GREEN} Container image ready${NC} ========"

    echo -e "======== ${YELLOW} Building all binaries in container${NC} ========"
    "$CONTAINER_RUNNER" run --rm \
        -v "${CARGO_REGISTRY_VOLUME_NAME}":/root/.cargo/registry:Z \
        -v "${REPO_DIR}":/build:z \
        -w /build \
        -v "${PACKAGE_DIR}":/packages:Z \
        "${IMAGE_TAG}" \
        /bin/bash -c "
            set -ex && \
            cd /build && ./wireguard/build-wireguard-go.sh && \
            cd /build/nym-vpn-core && \
            cargo build --package nym-vpnc --package nym-vpnd --release && \
            cd /build/nym-vpn-core/crates/test && \
            CARGO_TARGET_DIR=/build/nym-vpn-core/crates/test/target \
            cargo build --package test-runner --package connection-checker \
                --release --target x86_64-unknown-linux-gnu && \
            cp /build/nym-vpn-core/target/release/nym-vpnc /packages/ && \
            cp /build/nym-vpn-core/target/release/nym-vpnd /packages/ && \
            cp /build/nym-vpn-core/crates/test/target/x86_64-unknown-linux-gnu/release/test-runner /packages/ && \
            cp /build/nym-vpn-core/crates/test/target/x86_64-unknown-linux-gnu/release/connection-checker /packages/
        "
    echo -e "======== ${GREEN} All binaries built and copied to ${PACKAGE_DIR}${NC} ========"
}

function run_tests() {
    build_all_in_container

    pushd "${TEST_FRAMEWORK_ROOT}"
    cargo run \
        -p test-manager \
        -- run-tests \
        --vm ${NYM_TEST_VM_CONFIG} \
        --vnc 5901 \
        --nym-mnemonic "${MAINNET_MNEMONIC}" \
        --runner-dir "${PACKAGE_DIR}"\
        --verbose \
        "basic_functionality"
    popd
}

# Parse command-line arguments
if [ $# -eq 0 ]; then
    echo "Error: No command provided"
    echo ""
    usage
    exit 1
fi

check_required_vars

COMMAND="$1"

case "$COMMAND" in
    configure)
        configure
        ;;
    list)
        list
        ;;
    run-tests)
        run_tests
        ;;
    run-vm)
        run_vm
        ;;
    *)
        echo "Error: Unknown command '$COMMAND'"
        echo ""
        usage
        exit 1
        ;;
esac
