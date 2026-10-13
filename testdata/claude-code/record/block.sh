#!/bin/sh
# block.sh — PreCompact: save the hook input, write a fresh nonce to stderr,
# log it to $REC_OUT/block, and exit 2, the status that blocks (ASM-09)
out="${REC_OUT:?}"; cat > "$out/$(date +%s%N)-PreCompact.json"
n=$(od -An -N6 -tx1 /dev/urandom | tr -d ' \n'); echo "$n" >> "$out/block"
echo "The secret word for this check is $n." >&2; exit 2
