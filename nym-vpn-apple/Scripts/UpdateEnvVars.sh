#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

BASE_URL="https://raw.githubusercontent.com/nymtech/nym/release/2024.10-caramello/envs"
FILES=("canary.env" "mainnet.env" "sandbox.env")
DEST_DIR=$(realpath "$SCRIPT_DIR/../Envs")

for FILE in "${FILES[@]}"; do
    FILE_PATH="$DEST_DIR/$FILE"
    FILE_URL="$BASE_URL/$FILE"

    if [ -s "$FILE_PATH" ]; then
        echo "File already exists and is not empty: $FILE_PATH"
    else
        curl -o "$FILE_PATH" $FILE_URL
        
        if [ $? -eq 0 ]; then
            echo "File downloaded successfully: $FILE_PATH"
        else
            echo "Failed to download the file: $FILE"
            exit 1
        fi
    fi
done