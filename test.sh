#!/bin/bash -eu
cargo b
echo STARTING
(sleep 1; seq 2 5; seq 1 6) | while read line; do echo $line; sleep 1; done | ./target/debug/livesort
