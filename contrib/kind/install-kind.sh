#!/usr/bin/env bash
set -euo pipefail

VERSION="0.10.0"
LOCAL_BIN="$HOME/.local/bin"

if [ ! -d $LOCAL_BIN ]; then
    mkdir -p $LOCAL_BIN
fi

curl -LO "https://github.com/kubernetes-sigs/kind/releases/download/v$VERSION/kind-linux-amd64"

chmod +x kind-linux-amd64
mv kind-linux-amd64 "$LOCAL_BIN/kind"

echo "Downloaded 'kind' binary to $LOCAL_BIN"
