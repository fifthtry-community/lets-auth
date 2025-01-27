#!/bin/bash

cargo build --release --target wasm32-unknown-unknown

# Define the common source folder paths
SOURCE1="./target/wasm32-unknown-unknown/release/"
SOURCE2="$HOME/target/wasm32-unknown-unknown/release/"

# Define destination folders
DEST1="./app"
DEST2="./template/.packages/lets-talk.fifthtry.site"

# Ensure WASM files exist and determine the source folder to use
if [ -d "$SOURCE1" ]; then
    SOURCE_DIR=$SOURCE1
elif [ -d "$SOURCE2" ]; then
    SOURCE_DIR=$SOURCE2
else
    echo "Source folder not found."
    exit 1
fi

# Ensure the destination folders exist
mkdir -p $DEST1
mkdir -p $DEST2

# Copy files to destinations
cp "${SOURCE_DIR}mobile_auth_provider.wasm" "$DEST1"
cp "${SOURCE_DIR}email_auth_provider.wasm" "$DEST1"
cp "${SOURCE_DIR}mobile_auth_provider.wasm" "$DEST2"
cp "${SOURCE_DIR}email_auth_provider.wasm" "$DEST2"

echo "WASM files copied successfully."
