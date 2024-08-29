#!/bin/bash

BASE_URL="https://raw.githubusercontent.com/nymtech/nym/develop/envs"
FILES=("canary.env" "mainnet.env" "sandbox.env")
DEST_DIR="./Envs"

for FILE in "${FILES[@]}"; do
    FILE_URL="$BASE_URL/$FILE"

    curl -o "$DEST_DIR/$FILE" $FILE_URL
    
    if [ $? -eq 0 ]; then
        echo "File downloaded successfully: $DEST_DIR/$FILE"
    else
        echo "Failed to download the file: $FILE"
    fi
done