## Update Flatpak package

### Prerequisite

A new app **stable** release has been released and published

### Update the app metainfo

The app metainfo file is located at `.pkg/flatpak/net.nymtech.NymVPN.metainfo.xml`. \
Add a new `release` tag with the corresponding release info. \
Create a PR and merge into `develop`.

```xml
    <releases>
        <release version="1.2.3" date="2024-01-30">
            <url type="details">
                https://github.com/nymtech/nym-vpn-client/releases/tag/nym-vpn-app-v1.2.3
            </url>
        </release>
        <!-- other releases -->
    </releases>
```

### Update flatpak manifest

The package is hosted on [Flathub](https://flathub.org/apps/net.nymtech.NymVPN). \
The package repository is https://github.com/flathub/net.nymtech.NymVPN

Steps to update the package:

1. create a new branch from `master`
2. update the app manifest `net.nymtech.NymVPN.yml`

At the very least you need to update the `url` and `sha256` fields \
of the binary and the metainfo, to point to the new release.

```yaml
sources:
  - type: file
    # update the release tag and version
    url: https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-app-v1.2.3/nym-vpn_1.2.3_linux_x64
    sha256: xxxx # update the hash accordingly
    only-arches: [x86_64]
    dest-filename: nym-vpn
  - type: file
    # update the git hash to point to the new metainfo revision
    url: https://raw.githubusercontent.com/nymtech/nym-vpn-client/abcdef12/nym-vpn-app/.pkg/flatpak/net.nymtech.NymVPN.metainfo.xml
    sha256: xxxx # update the hash accordingly
```

3. submit the PR
4. a bot will trigger a build, if it is successful, check the package locally \
   and confirm it works as expected
5. CI is green
6. merge the PR

Once merged it will take some time for the package to be updated on Flathub.

ref https://docs.flathub.org/docs/for-app-authors/updates#creating-updates
