## Release

This doc describes how to release a new version of the NymVPN \
desktop app for Linux and Windows platforms

### Prerequisites

- Rust toolchain
- the targeted core `vpn-vpn-core-v*` version must be released
  and published

### Types of releases

Release tags must follow the following patterns depending on the release type:

- **stable** `nym-vpn-app-v1.2.3`
- rc `nym-vpn-app-v1.2.3-rc` for RC
- dev: `nym-vpn-app-v1.2.3-dev` or `nym-vpn-app-v1.2.3-beta` for beta builds
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

### Trigger the release

Go to the workflow
[publish-nym-vpn-app](https://github.com/nymtech/nym-vpn-client/actions/workflows/publish-nym-vpn-app.yml)
and click on the _Run workflow_ button

1. select the branch from which the release should be made \
   (including the version bump changes)

2. enter the release tag (including the version)

   **NOTE** refer to the [types of releases](#types-of-releases) section

3. select the release type
   - **stable** for a stable public release
   - **rc** for a release candidate (RC)
   - **dev** for a development release, like beta etc
   - **nightly** for a nightly build (not really used)

4. if **not** a stable release, you may want to label it as _Pre-release_

5. check _Enable updater_ if you want to enable the in-app updater (Windows only) \
   this is recommended for stable releases \
   if you check this:
   - the app will check for updates on startup, and if there is one available, it will prompt the user to update
   - it will create a PR to bump the updater JSON metadata
   - at build time, it will generate the bundle signature

6. check the _dev_ tickbox if it is a dev release, **not stable**, \
   and you want to enable the in-app dev menu

7. For stable releases, in the "nym-vpn-core release tag" input, enter the GH core release \
   tag that this app release targets, e.g. `nym-vpn-core-v1.2.3` \
   For dev releases, instead you can use any core dev builds using direct link to the Windows zip archive \
   just copy the url into the "direct link" input

   e.g. `https://.../nym-vpn-core-v1.2.3-beta_windows_x86_64.zip`

8. click the green _Run workflow_ button

If the release job is successful, the release has been published \
-> https://github.com/nymtech/nym-vpn-client/releases

---

### Post-release

Once the release is published, the artifacts and sources tarball are available \
for download from GitHub.

After publishing a **stable** release the workflow will update some packages automatically. \
But some post-release tasks still need to be done manually:

- see [update Flatpak](update_flatpak.md) to update the Flathub package
- bump `develop` to the next `-dev` version (if not already done)

#### nym.com

The website should automatically scrap any new stable release. \
Confirm all is looking as expected, e.g. download links, \
the displayed version and hash are correct

- https://nym.com/download/linux
- https://nym.com/download/windows

If not, it needs to be fixed in the nym-dot-com repo.
