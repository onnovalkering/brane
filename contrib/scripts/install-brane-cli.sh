#!/usr/bin/env bash
set -euo pipefail

# Determine the latest version (requires `jq`).
VERSION=$(\
    curl -L -s "api.github.com/repos/onnovalkering/brane/tags" \
  | jq -r '.[0].name' \
)

# Download the appropriate binary and save it as `brane`.
curl "github.com/onnovalkering/brane/releases/download/$VERSION/brane-`uname`" \
     -L -s -o brane

TARGET_DIR="$HOME/.local/bin"
mkdir -p $TARGET_DIR

# Add execute permissions and place it in the target directory.
chmod +x brane
mv brane $TARGET_DIR

# Check if target directory is in $PATH.
if [[ ! :$PATH: == *:"$TARGET_DIR":* ]] ; then
     echo "WARN: Please add '$TARGET_DIR' to \$PATH."
fi
