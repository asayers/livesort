#!/bin/bash -eu
cargo b

input () {
    (seq 2 5; seq 1 6) | while read line; do echo "The next number, if you'd believe it, really is $line"; done
}
slow() { while read line; do echo "$line"; sleep 1; done; }

echo "DIFF livesort vs sort"
diff <(input | ./target/debug/livesort) <(input | sort) && echo "OK"
echo "DIFF livesort -u vs sort | uniq"
diff <(input | ./target/debug/livesort -u) <(input | sort | uniq) && echo "OK"
echo "DIFF livesort -c vs sort | uniq -c"
diff <(input | ./target/debug/livesort -c) <(input | sort | uniq -c) && echo "OK"

echo
echo RUNNING livesort
(sleep 1; input) | slow | ./target/debug/livesort -c
