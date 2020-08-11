#!/bin/sh

set -e

INSTANTCLIENT="/etc/ld.so.conf.d/oracle-instantclient.conf"
LIBRARY="lofar_lta-2.7.1"

# Download and extract
curl http://www.astro-wise.org/losoft/$LIBRARY.tar.gz | tar xz && cd $LIBRARY

# Run installers
python3 setup.py --quiet install
python3 setup.py --quiet install_oracle

# LOFAR configuration
echo "[global]

; Database
database_user       : AWWORLD
database_password   : WORLD
database_engine     : oracle_10g
database_name       : db.lofar.target.rug.nl

; Server
data_server         : ds.lofar.target.astro-wise.org
data_port           : 8002
" > "$HOME/.awe/Environment.cfg"

# InstantClient configuration
if [ -d "/usr/lib/instantclient_11_2" ] ; then
    sh -c "echo /usr/lib/instantclient_11_2 > $INSTANTCLIENT" && ldconfig
else
    sh -c "echo /usr/local/lib/instantclient_11_2 > $INSTANTCLIENT" && ldconfig
fi

exit 0
