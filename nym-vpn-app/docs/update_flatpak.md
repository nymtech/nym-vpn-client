## Update Flatpak package

### Prerequisite

A new app **stable** release has been released and published

### How to update the Flatpak package

The package is hosted on [Flathub](https://flathub.org/apps/net.nymtech.NymVPN). \
The package repository is https://github.com/flathub/net.nymtech.NymVPN

Steps to update the package:

1. create a new branch from `master`
2. update the app manifest `net.nymtech.NymVPN.yml` accordingly to the new release

Most of the time it's needed to update the `url` and `sha256` fields \
of the app binary to point to the new release tag

3. submit the PR
4. a bot will trigger a build, if it is successful, check the package locally \
   and confirm it works as expected
5. CI is green
6. merge the PR

Once merged it will take some time for the package to be updated on Flathub.

ref https://docs.flathub.org/docs/for-app-authors/updates#creating-updates
