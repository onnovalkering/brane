#!/usr/bin/env bash
set -euo pipefail

export LC_ALL=C.UTF-8
export LANG=C.UTF-8

sed -i "s/PLC_SERVER_PORT/$SERVER_PORT/g" configuration.yml

vserver-local start --config "./configuration.yml"

echo "~~>output: done"
exit 0
