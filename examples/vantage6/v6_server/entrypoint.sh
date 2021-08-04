#!/usr/bin/env bash
set -euo pipefail

sed -i "s/PLC_SERVER_PORT/$SERVER_PORT/g" configuration.yml

# Run import inside virtual environment with Vantage6 v1.2.3 installed.
pipenv run vserver-local import --config ./configuration.yml --drop-all fixtures.yml
vserver-local start --config "./configuration.yml"

echo "~~>output: done"
exit 0
