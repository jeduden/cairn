#!/bin/sh
# tick.sh EVENT SECONDS — log start, then a tick every 100 ms until done or killed
ev="$1"; n=$(( $2 * 10 )); f="${REC_OUT:?}/tick-$ev-$(date +%s%N)"
cat > "$f.stdin"; date +%s%N > "$f"
i=0; while [ $i -lt $n ]; do sleep 0.1; date +%s%N >> "$f"; i=$((i+1)); done
echo done >> "$f"; exit 0
