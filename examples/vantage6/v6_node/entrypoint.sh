#!/usr/bin/env bash
set -euo pipefail

sed -i "s/PLC_API_KEY/$API_KEY/g" configuration.yml
sed -i "s/PLC_DATA_PATH/${DATA_PATH//\//\\/}/g" configuration.yml
sed -i "s/PLC_SERVER_HOST/$SERVER_HOST/g" configuration.yml
sed -i "s/PLC_SERVER_PORT/$SERVER_PORT/g" configuration.yml

# Generate random port number for the proxy.
export PROXY_SERVER_PORT=$(shuf -i 2000-65000 -n 1)

vnode-local start --config "./configuration.yml"

echo "~~>output: done"
exit 0
