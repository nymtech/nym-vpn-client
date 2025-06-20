#!/bin/bash

# this script is called by GH workflow `publish-nym-vpn-app.yml`
# it updates flatpak metainfo by inserting a new release block
# it expects the following environment variables to be set:
# - MATAINFO_XML, the path to the metainfo.xml file
# - VERSION, the app version eg. 1.2.3
# - RELEASE_TAG, the release tag eg. nym-vpn-app-v1.2.3
# - RELEASE_DATE, the date of the release in YYYY-MM-DD format (optional, defaults to today)

set -e

if [ -z "$MATAINFO_XML" ]; then
    echo "error MATAINFO_XML env variable not set"
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

# new release block to insert
rel_block="        <release version=\"$VERSION\" date=\"$date\">
            <url type=\"details\">
                https://github.com/nymtech/nym-vpn-client/releases/tag/$RELEASE_TAG
            </url>
        </release>"
rel_file=$(mktemp)
echo "$rel_block" > "$rel_file"

sed -i -e "/<!-- WF_ANCHOR_NEW_RELEASE -->/r $rel_file" "$MATAINFO_XML"
rm "$rel_file"

echo "✓ done"
