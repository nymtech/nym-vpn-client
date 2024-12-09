#!/usr/bin/env bash

# Run cargo metadata and use jq to filter for dependencies whose names start with "nym-"
# and have a git-based source. Then extract only the commit hash part of the URL.
cargo metadata --manifest-path nym-vpn-core/Cargo.toml --format-version=1 \
    | jq '.packages[]
    | select(
        (.name | startswith("nym-"))
        and ((.source // "") | contains("git+"))
      )
    | {name: .name, commit: (.source | split("#") | last)}'

