#!/usr/bin/env sh
if [ "$1" = "add" ]; then
    c=$(($A+$B))
fi

if [ "$1" = "substract" ]; then
    c=$(($A-$B))
fi

if [ "$1" = "multiply" ]; then
    c=$(($A*$B))
fi

if [ "$1" = "divide" ]; then
    c=$(($A/$B))
fi

echo "c: $c"
exit 0
