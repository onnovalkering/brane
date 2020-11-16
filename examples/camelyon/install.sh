#!/usr/bin/env bash
set -euo pipefail

pip install -r requirements.txt --no-cache-dir
git clone "https://github.com/medgift/PROCESS_L3.git"

cd "PROCESS_L3"

# Because 'n_samples' is twice in the config file, we prefix the first one (removed later).
sed -e '0,/n_samples/ s|n_samples|i_n_samples|' "doc/config.cfg"
