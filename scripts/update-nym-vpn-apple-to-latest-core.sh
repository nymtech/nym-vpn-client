#! /usr/bin/env bash
#
# Update nym-vpn-apple to use the latest release of nym-vpnd

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

APPLE_SCRIPTS_DIR=nym-vpn-apple/Scripts

# Parse arguments
for arg in "$@"; do
    case $arg in
        --yes)
        YES=true
        shift
        ;;
    esac
done

get_latest_core_version() {
    local repo="nymtech/nym-vpn-client"

    # Fetch the latest release using the GitHub API
    latest_release=$(curl -s "https://api.github.com/repos/$repo/releases/latest")

    # Extract the version tag from the JSON response
    latest_version=$(echo "$latest_release" | jq -r .tag_name)

    # Check if jq command was successful
    if [ $? -eq 0 ]; then
        echo "$latest_version"
    else
        echo "Failed to fetch the latest release version." >&2
        return 1
    fi
}

update_daemon_version_on_mac() {
    local core_version=$(get_latest_core_version)
    pushd "$APPLE_SCRIPTS_DIR"
    ./UpdateDaemonInfoPlist.sh $core_version
    popd
}

main() {
    check_unstaged_changes
    confirm_root_directory
    update_daemon_version_on_mac
}

main
