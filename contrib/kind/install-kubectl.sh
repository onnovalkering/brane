#!/usr/bin/env bash
set -euo pipefail

VERSION="1.20.5"
LOCAL_BIN="$HOME/.local/bin"

if [ ! -d $LOCAL_BIN ]; then
    mkdir -p $LOCAL_BIN
fi

curl -LO "https://storage.googleapis.com/kubernetes-release/release/v$VERSION/bin/linux/amd64/kubectl"

chmod +x kubectl
mv kubectl "$LOCAL_BIN/kubectl"

echo "Downloaded 'kubectl' binary to $LOCAL_BIN"
