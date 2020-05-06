#!/bin/bash -eu
cat Cargo.lock | while read line; do echo $line; sleep 0.2; done | cargo run
