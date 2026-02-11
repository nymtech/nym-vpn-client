# CI Setup for Test Harness

This document explains how to set up Continuous Integration (CI) for the NymVPN test harness.

## Overview

The test harness requires:
1. **System dependencies**: QEMU, rootlesskit, wireguard-tools, etc.
2. **Rust toolchain**: For building test binaries
3. **Test binaries**: `test-runner`, `connection-checker`
4. **Nym binaries**: `nym-vpnd`, `nym-vpnc`
5. **QCOW2 image**: A Linux VM image for testing

### Prerequisites

1. **QCOW2 Image**: You need a Linux QCOW2 image for testing. Options:
   - Download a pre-built image

2. **Secrets** (optional):
   - `MAINNET_MNEMONIC`: A mnemonic for testing (defaults to a test mnemonic if not set)

- Download QWOC2 from builds.ci.nymte.ch

### Environment Variables

The test harness uses these environment variables:

- `NYM_TEST_QCOW_IMAGE`: Path to QCOW2 image file
- `NYM_TEST_VM_CONFIG`: VM configuration name (default: `ci-test-vm`)
- `MAINNET_MNEMONIC`: Mnemonic for account testing
- `PACKAGE_DIR`: Directory containing built binaries

### Example Workflow Usage

```yaml
- name: Run test harness
  working-directory: nym-vpn-core/crates/test/test-manager
  env:
    NYM_TEST_QCOW_IMAGE: ~/.cache/nym-test/ubuntu-test.qcow2
    NYM_TEST_VM_CONFIG: ci-test-vm
    MAINNET_MNEMONIC: ${{ secrets.MAINNET_MNEMONIC }}
    PACKAGE_DIR: ~/.cache/nym-test/packages
  run: |
    cargo run --release -- run-tests \
      --vm "$NYM_TEST_VM_CONFIG" \
      --runner-dir "$PACKAGE_DIR" \
      --nym-mnemonic "$MAINNET_MNEMONIC" \
      --verbose \
      basic_functionality
```

## Self-Hosted Runners

1. Pre-install system dependencies
2. Pre-build and cache binaries
3. Store QCOW2 images locally
4. Configure persistent VM configurations

Example self-hosted runner setup:

```bash
# Install dependencies
sudo apt-get install -y \
  qemu-system-x86_64 qemu-utils \
  rootlesskit slirp4netns \
  wireguard-tools podman

# Pre-build binaries (optional)
cd nym-vpn-core/crates/test/test-manager
./test.sh run-tests basic_functionality
```
