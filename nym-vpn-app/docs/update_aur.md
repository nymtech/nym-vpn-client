## Update AUR packages

### Prerequisites

- A new app **stable** release has been released and published
- A new core **stable** release has been released and published

### How to update the AUR packages

We provide 2 AUR packages for the daemon:
- https://aur.archlinux.org/packages/nym-vpnd
- https://aur.archlinux.org/packages/nym-vpnd-bin

We provide 2 AUR packages for the app:
- https://aur.archlinux.org/packages/nym-vpn-app
- https://aur.archlinux.org/packages/nym-vpn-app-bin

To update them go to the workflow, respectively:
[publish-aur-nym-vpnd](https://github.com/nymtech/nym-vpn-client/actions/workflows/publish-aur-nym-vpnd.yml),
[publish-aur-nym-vpn-app](https://github.com/nymtech/nym-vpn-client/actions/workflows/publish-aur-nym-vpn-app.yml) \
and click on the _Run workflow_ button

1. select the branch from which the package update should be made,
   most of the time this should be `develop`

**NOTE** the only valid reasons to run this workflow from another branch are: \
Either you want to update the package files (i.e. `PKGBUILD`)
or you're updating the workflow itself

2. in the input _Tag name of the release_ enter the core or app release tag \
   e.g. `nym-vpn-core-v1.2.3` for vpnd, `nym-vpn-app-v1.2.3` for the app
3. _PKGBUILD package release number_ should remain set to `1` unless
   you know what you're doing
4. check _publish PKGBUILD changes to AUR_ for a **non** dry-run
5. _Commit message_ is optional, but good to pass the release version, e.g.
   `v1.2.3`
6. click the green _Run workflow_ button

If the job is successful, the corresponding AUR packages have been updated
