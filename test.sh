#!/bin/bash -eu
cargo b

input () {
    (seq 2 5; seq 1 6) | while read line; do echo "The next number, if you'd believe it, really is $line"; done
}
slow() { while read line; do echo "$line"; sleep 1; done; }

export LC_ALL=C
echo "DIFF livesort vs sort"
diff <(<Cargo.lock ./target/debug/livesort) <(<Cargo.lock sort) && echo "OK"
echo "DIFF livesort -u vs sort | uniq"
diff <(<Cargo.lock ./target/debug/livesort -u) <(<Cargo.lock sort | uniq) && echo "OK"
echo "DIFF livesort -c vs sort | uniq -c"
diff <(<Cargo.lock ./target/debug/livesort -c) <(<Cargo.lock sort | uniq -c) && echo "OK"

echo
echo RUNNING livesort
(sleep 1; input) | slow | ./target/debug/livesort -c
