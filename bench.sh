#!/bin/bash -eu
cargo b --release
export LC_ALL=C
FILE=Cargo.lock
livesort_installed="livesort -c <$FILE"
livesort_head="./target/release/livesort -c <$FILE"
sort="sort <$FILE | uniq -c"
cmp <(eval $livesort_head) <(eval $sort) && echo "sanity check (HEAD): ok" >&2
cmp <(eval $livesort_installed) <(eval $sort) && echo "sanity check (installed): ok" >&2
cbdr sample "sort:$sort" "livesort_installed:$livesort_installed" "livesort_HEAD:$livesort_head"
