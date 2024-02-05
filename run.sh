#!/bin/bash -e
# script to easily start all assyst processes
# not suitable for production but handy when testing

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

cargo b -p assyst-gateway
cargo b -p assyst-cache
cargo b -p assyst-core

cargo r -p assyst-gateway &
P1=$!
cargo r -p assyst-cache &
P2=$!
cargo r -p assyst-core &
P3=$!
wait $P1 $P2 $P3