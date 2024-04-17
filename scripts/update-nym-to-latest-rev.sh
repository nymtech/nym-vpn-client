#!/bin/bash

# File to update
cargo_file="./Cargo.toml"

# Check if the Cargo.toml file exists
if [ ! -f "$cargo_file" ]; then
    echo "File: "$cargo_file" does not exist!"
    exit 1
fi

# GitHub API URL for the latest commit on the develop branch
api_url="https://api.github.com/repos/nymtech/nym/commits/develop"

# Fetch the latest commit SHA from the develop branch
latest_commit=$(curl -s $api_url | jq -r '.sha' | cut -c 1-7)

# Check if we got a valid commit SHA
if [[ -z "$latest_commit" || "$latest_commit" == "null" ]]; then
    echo "Failed to fetch the latest commit SHA. Exiting..."
    exit 1
fi

echo "Latest commit SHA: $latest_commit"

# Update the Cargo.toml file with the latest commit SHA
sed -i -E "s/(nym-.* = \{ git = \"https:\/\/github\.com\/nymtech\/nym\", rev = \")([a-f0-9]+)/\1$latest_commit/" "$cargo_file"

if [ $? -eq 0 ]; then
    echo "Updated Cargo.toml with the latest commit SHA: $latest_commit"
else
    echo "Failed to update Cargo.toml"
fi

