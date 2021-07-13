#!/usr/bin/env bash
set -euo pipefail
shopt -s nocasematch

if [[ $1 == "listen" ]]; then
    if [[ $KEEP_ALIVE == "TRUE" ]]; then
        nc -k -l $PORT
    else
        nc -l $PORT
    fi
else
    echo $MESSAGE | nc -q0 $ADDRESS $PORT
fi

echo "~~>output: done"
exit 0
