# Nym VPN End-to-End Test Framework

Forked from [Mullvad VPN](https://github.com/mullvad/mullvadvpn-app) under
GPL-3.0-only.

## Architecture

This framework runs end-to-end tests against `nym-vpnd` (the VPN daemon) inside
a guest VM. A host-side orchestrator (**test-manager**) launches the VM, deploys
binaries, and drives tests over a virtual serial port. The serial port is used
instead of the network because the VPN tunnel redirects traffic, making SSH
unreliable during tests.

### Nested VMs

Both CI and local runs use a **VM-inside-a-VM** layout:

```
┌──────────────────────────────────────────────────────┐
│  GitHub Actions Runner / Developer Machine           │
│                                                      │
│  builds binaries, provisions outer VM via Terraform  │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │  Exoscale VM  (Ubuntu, KVM-capable)            │  │
│  │   (runs) test-manager                          │  │
│  │ ┌───────────┘                                  │  │
│  │ │                                              │  │
│  │ │ ┌─────────────────────────────────────────┐  │  │
│  │ │ │  QEMU Guest VM  (Debian 12)             │  │  │
│  │ │ │                                         │  │  │
│  │ │ │  runs test-runner, nym-vpnd, nym-vpnc   │  │  │
│  │ │ │                                         │  │  │
│  │ │ │                                         │  │  │
│  │ └─── (serial port) <-> test-manager         │  │  │
│  │   │                                         │  │  │
│  │   └─────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

Outer VM is required if the machine running test-manager doesn't support QEMU or
KVM. Exoscale VM provides this. On a Linux host with KVM you could skip the
outer VM and run `test-manager` directly, but the Exoscale path gives a
consistent, reproducible environment for CI and macOS developers who cannot run
QEMU locally (efficiently).

## Crate structure

| Crate                  | Runs on         | Purpose                                                                                                |
| ---------------------- | --------------- | ------------------------------------------------------------------------------------------------------ |
| **test-manager**       | Host / outer VM | Orchestrates tests, manages VM lifecycle, serial-port client. Tests live in `test-manager/src/tests/`. |
| **test-runner**        | Guest VM        | Exposes RPCs for the test-manager to control the VPN daemon, network, filesystem, and processes.       |
| **test-rpc**           | Shared library  | RPC interface (`#[tarpc::service]`), serial transport, and common types.                               |
| **test_macro**         | Compile-time    | Proc macro providing `#[test_function_nym]` for test registration.                                     |

### Communication model

All host-to-VM communication goes through a **single serial port** with
frame-based multiplexing (`test-rpc/src/transport.rs`):

1. **TestRunner channel** (`Frame::TestRunner`): tarpc RPC for OS-level test
   operations (install app, manage daemon service, send packets, etc.)
2. **DaemonRpc channel** (`Frame::DaemonRpc`): gRPC forwarding to `nym-vpnd`
   inside the VM via Unix socket


## CI (`e2e-test.yml`)

The GitHub Actions workflow has two jobs:

1. **`build`**: On an Ubuntu runner: installs toolchains, builds `nym-vpnc`,
   `nym-vpnd`, `test-manager`, and `test-runner`, then
   uploads them as an artifact.
2. **`e2e-test`**: On a second runner:
   - Provisions an Exoscale VM via Terraform (`ci/exoscale.tf`)
   - SCPs the binaries + `test-manager/test.sh` to the VM
   - Runs `test.sh configure` then `test.sh run-tests` over SSH
   - Collects the test report, then destroys the VM

Workflow inputs allow filtering/skipping tests and delaying teardown for SSH
debugging.


## Building

```bash
# Full containerized build (recommended, produces Linux x86_64 binaries)
# From the repo root:
./nym-vpn-core/crates/test/test-manager/test_macos.sh

```

The Dockerfile (`Dockerfile`) builds a Debian 12 image with Rust, Go, static
libpcap, and static OpenSSL. Debian 12 matches the guest VM environment to avoid
dynamic linking issues.

## Writing tests

Tests are async functions annotated with `#[test_function_nym]`:

```rust
#[test_function_nym]
pub async fn my_test(
    _: TestContext,
    rpc: NymServiceClient,           // tarpc client to test-runner in VM
    mut nym_client: NymProxyClient,   // gRPC client to nym-vpnd in VM
) -> Result<(), anyhow::Error> {
    Ok(())
}
```

Macro attributes: `priority = <i32>` (lower runs first),
`target_os = "linux"|"macos"|"windows"`.

New test modules must be added to `test-manager/src/tests/mod.rs`.
