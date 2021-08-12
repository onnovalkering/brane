#!/usr/bin/env bash
set -euo pipefail

sed -i "s/PLC_SERVER_PORT/$SERVER_PORT/g" configuration.yml

# Run import inside virtual environment with Vantage6 v1.2.3 installed.
pipenv run vserver-local import --config ./configuration.yml --drop-all fixtures.yml

# Disable encryption for existing collaborations.
pipenv run python -c " 
from vantage6.cli.context import ServerContext;
from vantage6.server.model.base import Database;

ctx = ServerContext.from_external_config_file('./configuration.yml', 'prod', True);
Database().connect(ctx.get_database_uri());

import vantage6.server.db as db;
for c in db.Collaboration.get(): c.encrypted = False; c.save();
"

# Start the server.
vserver-local start --config "./configuration.yml"

echo "~~>output: done"
exit 0