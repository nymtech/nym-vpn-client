#!/bin/bash

# this script is called by GH workflow `publish-nym-vpn-app.yml`
# it updates flatpak metainfo by inserting a new release block
# it expects the following environment variables to be set:
# - METAINFO_XML, the path to the metainfo.xml file
# - VERSION, the app version eg. 1.2.3
# - RELEASE_TAG, the release tag eg. nym-vpn-app-v1.2.3
# - RELEASE_DATE, the date of the release in YYYY-MM-DD format (optional, defaults to today)

set -e

if [ -z "$METAINFO_XML" ]; then
    echo "error METAINFO_XML env variable not set"
    exit 1
fi
if [ -z "$VERSION" ]; then
    echo "error VERSION env variable not set"
    exit 1
fi
if [ -z "$RELEASE_TAG" ]; then
    echo "error RELEASE_TAG env variable not set"
    exit 1
fi

date="$RELEASE_DATE"
if [ -z "$date" ]; then
    echo "RELEASE_DATE env variable not set, using today's date"
    date=$(date -u +'%Y-%m-%d')
fi

xmlstarlet edit --inplace \
    --insert '//component/releases/node()[1]' --type elem -n 'release' \
    --var new_node '$prev' \
    --insert '$new_node' --type attr -n 'version' --value "$VERSION" \
    --insert '$new_node' --type attr -n 'date' --value "$date" \
    --subnode '$new_node' --type elem -n 'url' -v "https://github.com/nymtech/nym-vpn-client/releases/tag/$RELEASE_TAG" \
    --var new_node '$prev' \
    --insert '$new_node' --type attr -n 'type' --value 'details' \
    "$METAINFO_XML"
formatted_xml=$(xmlstarlet format --indent-spaces 4 "$METAINFO_XML")
echo "$formatted_xml" > "$METAINFO_XML"

echo "✓ done"
