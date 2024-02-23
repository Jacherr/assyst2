#!/bin/bash -e
# script to easily start all assyst processes
# not suitable for production but handy when testing

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

mold -run cargo b -p assyst-gateway
mold -run cargo b -p assyst-cache
mold -run cargo b -p assyst-core

cargo r -p assyst-cache &
P1=$!
# allow cache to start first
sleep 0.5
cargo r -p assyst-core &
P2=$!
# allow core and cache to sync before sending events
sleep 0.5
cargo r -p assyst-gateway &
P3=$!
wait $P1 $P2 $P3