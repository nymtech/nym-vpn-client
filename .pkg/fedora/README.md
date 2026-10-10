# Fedora 44 RPM packaging for NymVPN 2026.11.2

This directory builds an unsigned, all-in-one Fedora 44 RPM for NymVPN 2026.11.2 on `x86_64` and `aarch64`. It is practical distribution packaging rather than a Fedora-review submission.

The package follows Nym's current Linux component layout while using the combined-package model used by Mullvad: the GUI, daemon, CLI, diagnostic tool, and daemon helpers share one RPM lifecycle.

## Pinned inputs

The build uses the upstream tag `nym-vpn-v2026.11.2` at commit `cd8606d29d25416ed67a93d718c44ff65356ddfc`. The GitHub source archive is accepted only when its SHA-256 is:

```text
6d999fce5a83027aaccc71880f12a61ed2b74be6a38d70f0c0258d317c608463
```

The Fedora builder pins Rust 1.95.0, Go 1.24.4, Node.js 24.18.0, and protoc 21.12. Their architecture-specific archive checksums are recorded in `sources.lock`. Cargo and npm are allowed to download dependencies selected by the upstream lockfiles. The build does not inject private Sentry DSNs, and the GUI updater and development mode are disabled.

## Build on macOS

From the repository root, enter the packaging directory:

```bash
cd .pkg/fedora
```

Initialize a dedicated Podman VM once:

```bash
scripts/bootstrap-podman-macos.sh
```

When newly created, the VM uses 8 CPUs, 12 GiB RAM, 12 GiB swap, and a 100 GiB disk. On Apple silicon the bootstrap also enables Rosetta for the amd64 build without changing the user's global Podman configuration. The script is safe to rerun after restarting the Mac or VM. Then build one or both architectures:

```bash
scripts/build-rpm.sh aarch64
scripts/build-rpm.sh x86_64
scripts/build-rpm.sh all
```

On Apple silicon, `aarch64` builds natively and `x86_64` uses Podman's amd64 emulation. The latter is substantially slower because the Rust release build uses LTO.

If an existing `nym-rpm-builder` predates these resource settings, remove and recreate that build-only VM or adjust its CPU, memory, and disk allocation before building. Swap is fixed at machine creation time. Run the bootstrap again whenever the VM needs to be started so Podman applies this project's Rosetta setting.

Outputs are written below `dist/`:

```text
dist/
├── aarch64/
│   ├── nym-vpn-2026.11.2-1.fc44.aarch64.rpm
│   ├── build.log
│   ├── validation.log
│   └── SHA256SUMS
├── x86_64/
│   └── ...
├── SRPMS/
│   └── nym-vpn-2026.11.2-1.fc44.src.rpm
└── SHA256SUMS
```

The build wrapper accepts `CONTAINER_ENGINE=docker` for compatible Linux environments. Native dual-architecture jobs are defined in `.github/workflows/build-fedora-rpm.yml`; CI uploads artifacts but does not sign, release, publish, or create a repository. Generated RPMs, logs, and dependency caches are ignored by Git and are not committed to the source repository.

## Installed interface

The RPM installs:

- `/usr/bin/nym-vpn-app`
- `/usr/bin/nym-vpnd`
- `/usr/bin/nym-vpnc`
- `/usr/bin/nym-exclude`
- `/usr/bin/nym-socks5-proxy`
- `/usr/bin/nym-diagnostic`
- `nym-vpnd.service`, a polkit action, a desktop entry with the `nymvpn://` deep-link handler, a scalable icon, and AppStream metadata

`nym-exclude` is deliberately installed as `root:root` mode `4755`, matching Nym's upstream Linux packaging design. It places an invoked process into the split-tunnel exclusion cgroup, drops privileges back to the calling user, and executes the requested command. Treat changes to this helper as security-sensitive.

On first installation, the RPM enables and starts `nym-vpnd.service`. An upgrade restarts an active daemon but does not re-enable one that the user disabled. Final removal stops and disables the daemon. `/etc/nym`, `/var/lib/nym-vpnd`, and `/var/log/nym-vpnd` are intentionally not owned or removed by the RPM, so uninstalling does not destroy user state.

Install an artifact with:

```bash
sudo dnf install ./dist/x86_64/nym-vpn-2026.11.2-1.fc44.x86_64.rpm
```

The RPMs are unsigned. Verify `dist/SHA256SUMS` before installation.

## Validation

Every architecture build performs these checks in a clean Fedora 44 builder:

- source and artifact digest verification
- `rpmlint`, RPM metadata/dependency inspection, and required-payload checks
- desktop entry, AppStream, polkit XML, icon, and systemd unit validation
- DNF installation and RPM verification
- shared-library resolution for every executable
- version checks for the public commands and package-ownership version checks for protocol-only helpers
- setuid ownership/mode verification
- a GUI startup smoke test under Xvfb

`scripts/systemd-smoke.sh` uses a disposable privileged Fedora 44 systemd container to test first-install activation, CLI-to-daemon communication, disabled-service preservation across reinstall, final removal, and state-directory preservation. It deliberately does not establish a VPN tunnel because doing so requires credentials and changes live networking.

The public release tag contains English UI strings only. This first RPM is therefore English-only.
