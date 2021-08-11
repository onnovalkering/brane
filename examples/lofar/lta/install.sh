#!/bin/sh

set -e

#~~ SRM ~~#

SRM_CLIENT="srmclient-2.6.28"

echo "deb [trusted=yes] https://repository.egi.eu/sw/production/cas/1/current egi-igtf core" >> /etc/apt/sources.list
wget -q -O - https://repository.egi.eu/sw/production/cas/1/current/GPG-KEY-EUGridPMA-RPM-3 | apt-key add -

apt-get update && apt-get install -y ca-policy-egi-core && rm -rf /var/lib/apt/lists/*

curl https://www.astron.nl/lofarwiki/lib/exe/fetch.php?media=public:$SRM_CLIENT.tar.gz | tar xz

mkdir -p /etc/grid-security/vomsdir/lofar
mv voms.grid.sara.nl.lsc /etc/grid-security/vomsdir/lofar

mkdir /etc/vomses
mv lofar.vo /etc/vomses

#~~ LTA ~~#

INSTANTCLIENT="/etc/ld.so.conf.d/oracle-instantclient.conf"
LTA_CLIENT="lofar_lta-2.7.1"

curl https://www.astro-wise.org/losoft/$LTA_CLIENT.tar.gz | tar xz && cd $LTA_CLIENT

python3 setup.py --quiet install
python3 setup.py --quiet install_oracle

mv ../Environment.cfg "$HOME/.awe/Environment.cfg"

if [ -d "/usr/lib/instantclient_11_2" ] ; then
    sh -c "echo /usr/lib/instantclient_11_2 > $INSTANTCLIENT" && ldconfig
else
    sh -c "echo /usr/local/lib/instantclient_11_2 > $INSTANTCLIENT" && ldconfig
fi

exit 0
