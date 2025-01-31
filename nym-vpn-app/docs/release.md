## Release

This doc describes how to release a new version of the NymVPN \
desktop app for Linux and Windows platforms

### Prerequisites

- Rust toolchain
- the targeted core `vpn-vpn-core-v*` version must be released
  and published

### Types of releases

Release tags must follow the following patterns:

- **stable** `nym-vpn-app-v1.2.3`
- dev `nym-vpn-app-v1.2.3-dev`
- RC `nym-vpn-app-v1.2.3-rc.1`
- nightly `nym-vpn-app-nightly`

### Bump versions

1. update the version in the `src-tauri/Cargo.toml` \
   e.g. if the release version is `1.2.3`

```toml
version = "1.2.3"
```

`src-tauri/Cargo.lock` should be updated accordingly, \
run `cargo build` and recheck the `Cargo.lock` changes.

2. update the vpnd compatibility semver version
   [requirement](https://docs.rs/semver/1.0.23/semver/struct.VersionReq.html) \
   edit the file `vpnd_compat.toml` \
   e.g. if this app release is compatible with any vpnd versions >= `1.2.0`

```toml
version = ">=1.2.0"
```

3. in the same way update the vpnd compatibility for the deb package \
   edit the property `linux.deb.depends` in `src-tauri/tauri.conf.json`

```
"depends": ["nym-vpnd (>= 1.2.0)"],
```

4. push the changes to the repository (likely via a dedicated
   branch)

---

There are 2 ways to release the app:

1. manually from the GH workflow, recommended for now
2. via git tag (_TODO_)

### Trigger the release manually

Go to the workflow
[publish-nym-vpn-app](https://github.com/nymtech/nym-vpn-client/actions/workflows/publish-nym-vpn-app.yml)
and click on the _Run workflow_ button

1. select the branch from which the release should be made \
   (including the version bump changes)

2. enter the release tag (including the version)

   **NOTE** refer to the [types of releases](#types-of-releases) section

3. :warning: if it is **not** a stable release check the _Pre-release_ tickbox

4. check the _dev_ tickbox if it is a dev release, not stable, \
   (enable in-app dev menu)

5. in the _nym-vpn-core release tag_ input, enter the core release \
   tag that this app release targets, e.g. `nym-vpn-core-v1.2.3`

6. click the green _Run workflow_ button

If the release job is successful, the release has been published \
-> https://github.com/nymtech/nym-vpn-client/releases

### Trigger the release via a git tag

_TODO_

---

### Post-release

Once the release is published, the artifacts and sources tarball are available \
for download from GitHub. \
Generally after a **stable** release we want to update our different packages \
to distribute the new version.

#### Linux packages

For now, except the `deb` package which is automatically updated \
on new stable, the rest of the packages need to be updated manually.

- refer to [update AUR](update_aur.md) for Arch Linux packages
- refer to [update Flatpak](update_flatpak.md) for Flatpak package

#### nym.com

The website should automatically scrap any new stable release. \
Confirm all is looking as expected, e.g. download links, \
the displayed version and hash are correct

- https://nym.com/download/linux
- https://nym.com/download/windows

If not, it needs to be fixed in the nym-dot-com repo.
