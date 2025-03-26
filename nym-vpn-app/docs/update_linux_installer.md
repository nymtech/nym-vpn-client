## Update Linux installer script

### Prerequisite

A new app **stable** release has been released and published

### Update

Bump the versions in the script `.pkg/linux/install`:

```bash
# AppImage
app_tag=nym-vpn-app-v1.2.3
app_version=1.2.3
# […]
# nym-vpnd
vpnd_tag=nym-vpn-core-v1.2.3
vpnd_version=1.2.3
```

Push the changes to `develop` branch.
