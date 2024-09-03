#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

BASE_URL="https://raw.githubusercontent.com/nymtech/nym/release/2024.10-caramello/envs"
FILES=("canary.env" "mainnet.env" "sandbox.env")
DEST_DIR=$(realpath "$SCRIPT_DIR/../Envs")

for FILE in "${FILES[@]}"; do
    FILE_URL="$BASE_URL/$FILE"

    curl -o "$DEST_DIR/$FILE" $FILE_URL
    
    if [ $? -eq 0 ]; then
        echo "File downloaded successfully: $DEST_DIR/$FILE"
    else
        echo "Failed to download the file: $FILE"
        exit 1
    fi
done