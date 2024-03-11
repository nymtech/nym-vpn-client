#! /usr/bin/env bash
#
# Bump the version of nym-vpn-desktop and tag it.
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

# Confirm we are in the root of the directory

pushd nym-vpn-desktop
pushd src-tauri
# Bump the patch level version
command="cargo set-version --bump patch"
# npm version patch
# cargo set-version 0.0.6-dev
# npm version 0.0.6-dev

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

popd

# We can't run this with --dry-run, so let's assume it will be fine
command_npm="npm version patch"
echo "Running: $command_npm"
$command_npm

# Read the new version
# version=$(cargo get package.version --entry="nym-vpn-desktop")
version=$(cargo get package.version --entry src-tauri)
tag_name=nym-vpn-desktop-v$version

# Tag the release
read -p "Do you want to tag this commit with: $tag_name ? (Y/N): " confirm_tag
if [[ $confirm_tag =~ ^[Yy]$ ]]; then
    echo "Tagging the commit with tag: $tag_name"
    git commit -a -m "Bump nym-vpn-desktop to $version"
    git tag $tag_name
    # Optionally, push the tag to remote repository
    # git push origin $tag
else
    echo "Not tagging."
fi
