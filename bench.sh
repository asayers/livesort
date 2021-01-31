#!/bin/bash -eu
cargo b --release
export LC_ALL=C
FILE=Cargo.lock
livesort="./target/release/livesort -c <$FILE"
sort="sort <$FILE | uniq -c"
cmp <(eval $livesort) <(eval $sort) && echo "sanity check ok" >&2
cbdr sample "livesort:$livesort" "sort:$sort"
