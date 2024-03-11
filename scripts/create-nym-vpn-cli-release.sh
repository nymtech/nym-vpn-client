#! /usr/bin/env bash
#
# Bump the version of nym-vpn-cli and tag it.
# Pushing this upstream will then trigger a new release.
#
# Currently it's always the patch version that is bumped, if you need to bump
# another version you have to do it manually, which is a good thing, so that we
# don't bump on accident.

# set -x
set -euo pipefail

# Confirm we don't have unstaged changes
if ! git diff --exit-code > /dev/null; then
    echo "Error: There are unstaged changes. Please commit or stash them before running this script."
    exit 1
fi

# Bump the patch level version
command="cargo set-version -p nym-vpn-cli --bump patch"

# Run the command with --dry-run option first
echo "Running in dry-run mode: $command --dry-run"
$command --dry-run

# Ask for user confirmation
read -p "Was this the intended change? (Y/N): " answer
if [[ $answer =~ ^[Yy]$ ]]; then
    echo "Running command without dry-run: $command"
    $command
else
    echo "Exiting without making changes."
    exit 1
fi

# Read the new version
version=$(cargo get package.version --entry="nym-vpn-cli")
tag_name=nym-vpn-cli-v$version

# Tag the release
read -p "Do you want to tag this commit with: $tag_name ? (Y/N): " confirm_tag
if [[ $confirm_tag =~ ^[Yy]$ ]]; then
    echo "Tagging the commit with tag: $tag_name"
    git commit -a -m "Bump nym-vpn-cli to $version"
    git tag $tag_name
    # Optionally, push the tag to remote repository
    # git push origin $tag
else
    echo "Not tagging."
fi
