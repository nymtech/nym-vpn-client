#!/bin/bash

# this script is called by GH workflow `publish-nym-vpn-app.yml`
# it updates the install script by bumping app and core versions
# (stable versions only)
# it expects the following environment variables to be set:
# - APP_TAG, eg. nym-vpn-app-v1.2.3 (or the unified tag nym-vpn-v1.2.3)
# - CORE_TAG is the unified release tag (nym-vpn-v<version>) under unified-only core.

set -e

# Accept the legacy app tag (nym-vpn-app-v*) and the unified tag (nym-vpn-v*).
# Under unified-only releases APP_TAG is the unified tag and app == core version.
case "$APP_TAG" in
  nym-vpn-app-v*) app_version=${APP_TAG#nym-vpn-app-v} ;;
  nym-vpn-v*)     app_version=${APP_TAG#nym-vpn-v} ;;
  *) echo "wrong app tag: $APP_TAG"; exit 1 ;;
esac
if ! echo "$CORE_TAG" | grep -q '^nym-vpn-v'; then
  echo "wrong core tag: $CORE_TAG"
  exit 1
fi

core_version=${CORE_TAG#nym-vpn-v}
echo "app version: $app_version"
echo "core version: $core_version"

if [ -z "$app_version" ]; then
  echo "failed to extract app version from tag: $APP_TAG"
  exit 1
fi
if [ -z "$core_version" ]; then
  echo "failed to extract core version from tag: $CORE_TAG"
  exit 1
fi

sed -i "s|^app_tag=.*|app_tag=$APP_TAG|" install
sed -i "s|^app_version=.*|app_version=$app_version|" install
sed -i "s|^vpnd_tag=.*|vpnd_tag=$CORE_TAG|" install
sed -i "s|^vpnd_version=.*|vpnd_version=$core_version|" install

echo "✓ done"
