# Local development

There are 2 ways to run this nested VM setup. In both cases, you've got a single script as the entrypoint to run the whole end-to-end suite.

## Ubuntu/Debian

You need to have KVM compliant qemu installation. In that case, the flow is the following
- **[your PC]** `./test.sh run-tests` ->(spawns)-> **[guest VM inside qemu]** `test-runner`, `nym-vpnd`

```
┌──────────────────────────────────────────────────────┐
│        Developer Machine (KVM capable via qemu)      │
│                   builds binaries                    │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │    qemu VM  (Ubuntu)                           │  │
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

Explanation:
- prepare a QCOW2 image that will be used as a base for the test VM
  - image needs to have proper packages installed, user/password set up etc.
  - ask any of the developers who worked on it to provide you with one
- `./test.sh configure`: creates the config
- `./test.sh run-tests`: runs test suite with above config
- in this case, `qemu` guest on your machine acts as a host for `test-manager` (see diagram above)

If you encounter package or env variable errors, fix those as you go. It doesn't make sense to list every bit of configuration here because those things change over time, so we avoid this document becoming stale.

### Requirements

[install_deps_linux.sh](../scripts/install_deps_linux.sh)

## MacOS (Apple silicon)
That means you DON'T* have kvm compliant qemu installation so you CANNOT run nested VM via `qemu` on your host.

- **[your PC]** `./test_macos.sh` ->(provisions) -> **[exoscale]** Ubuntu VM ->(spawns)-> **[guest VM inside qemu]** `test-runner`, `nym-vpnd`



```
┌──────────────────────────────────────────────────────┐
│           Developer Machine (non-KVM capable)        │
│             has exoscale credentials                 │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │            Exoscale VM (Ubuntu)                │  │
│  │            (runs) test-manager                 │  │
│  │ ┌────────────────────┘                         │  │
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


Explanation:
- prepare exoscale credentials
- set necessary env vars required by `test_macos.sh`
- `./test_macos.sh` will spawn an exoscale remote VM, which will act as a host for test-runner (refer to diagram above)

### Requirements

- Docker or Podman
- Terraform / OpenTofu
- Exoscale account with API credentials and a pre-provisioned data volume
  containing the QCOW2 guest image (already exists, you just provide the ID of the resource)
  - used as a blank slate for each test, not modified during test runs


>\* In theory, qemu is available on MacOS, but hardware virtualization of an x86 guest is impossible on ARM so in practice you don't have it. Remember, you're running VM inside a VM.

## `test.sh`: the common entrypoint

`test-manager/test.sh` is the script that actually invokes `test-manager`. It
works in two modes:

- **Local mode** (repo checkout exists): builds binaries in a container, then
  runs `test-manager` via `cargo run`.
- **CI/remote mode** (`TEST_DIST_DIR` is set): uses pre-built binaries from the
  dist directory.
